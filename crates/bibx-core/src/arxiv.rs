//! Fetch source bundles from arXiv and select a submitted bibliography.

mod archive;

use std::fs::{self, File};
use std::io::{self, Seek};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use directories::ProjectDirs;
use tempfile::NamedTempFile;

use crate::identifier::{Kind, is_arxiv_id, url_identifier};

const BASE_URL: &str = "https://arxiv.org";
const DOWNLOAD_LIMIT: u64 = 100 * 1024 * 1024;

type Result<T> = std::result::Result<T, PullError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArxivId(String);

impl ArxivId {
    fn versioned(&self) -> bool {
        self.0
            .rsplit('/')
            .next()
            .is_some_and(|number| number.contains('v'))
    }
}

impl FromStr for ArxivId {
    type Err = PullError;

    fn from_str(input: &str) -> Result<Self> {
        let value = if let Some(url) = input.strip_prefix("url:") {
            match url_identifier(url) {
                Some((Kind::Eprint, id)) => id,
                _ => return Err(PullError::InvalidId),
            }
        } else if let Some((kind, id)) = input.split_once(':') {
            if kind.eq_ignore_ascii_case("arxiv") || kind == "eprint" {
                id
            } else {
                return Err(PullError::InvalidId);
            }
        } else {
            input
        };
        if !is_arxiv_id(value) {
            return Err(PullError::InvalidId);
        }
        Ok(Self(value.to_owned()))
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PullError {
    #[error("expected an arXiv identifier or a url: prefixed arXiv abstract or PDF link")]
    InvalidId,
    #[error("--path must be a relative archive path with / separators")]
    InvalidMember,
    #[error("source is unavailable for arXiv:{0}")]
    NoSource(String),
    #[error("arXiv source download returned HTTP {0}")]
    Http(u16),
    #[error("could not download arXiv source")]
    Download(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("could not prepare a temporary source file")]
    TempFile(#[source] io::Error),
    #[error("unsupported arXiv source format (expected a gzipped tar bundle or TeX file)")]
    UnsupportedSource,
    #[error("invalid arXiv source archive")]
    InvalidArchive(#[source] io::Error),
    #[error("{0} exceeds the limit of {1}")]
    Limit(&'static str, &'static str),
    #[error("several .bib files found, pass --path to choose one:{}", archive::list_paths(.0))]
    Ambiguous(Vec<Vec<u8>>),
    #[error("archive contains more than one .bib member at {0}")]
    DuplicateMember(String),
    #[error("no .bib member matches {0}")]
    NoMember(String),
    #[error("the submission contains a .bbl file but no .bib file")]
    BblOnly,
    #[error("the submission contains no .bib or .bbl file")]
    NoBibliography,
}

pub struct Arxiv {
    agent: ureq::Agent,
    base_url: String,
    cache: Option<PathBuf>,
}

impl Arxiv {
    pub fn new() -> Self {
        #[cfg(feature = "testing")]
        if let Ok(url) = std::env::var("BIBX_ARXIV_TEST_URL") {
            let cache = std::env::var_os("BIBX_ARXIV_TEST_CACHE").map(PathBuf::from);
            return Self::with_config(&url, cache);
        }

        let cache = ProjectDirs::from("", "", "bibx")
            .map(|dirs| dirs.cache_dir().join("arxiv").join("source"));
        Self::with_config(BASE_URL, cache)
    }

    fn with_config(base_url: &str, cache: Option<PathBuf>) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .user_agent(concat!("bibx/", env!("CARGO_PKG_VERSION")))
            .https_only(base_url.starts_with("https://"))
            .http_status_as_error(false)
            .build()
            .into();
        Self {
            agent,
            base_url: base_url.trim_end_matches('/').into(),
            cache,
        }
    }

    /// Return the selected member byte for byte. Member paths use archive separators.
    pub fn pull(&self, id: &ArxivId, member: Option<&Path>) -> Result<Vec<u8>> {
        let member = member
            .map(|path| {
                archive::member_path(path.as_os_str().as_encoded_bytes())
                    .ok_or(PullError::InvalidMember)
            })
            .transpose()?;
        let cached = self.cache_path(id);
        if let Some(path) = &cached
            && fs::symlink_metadata(path).is_ok_and(|meta| meta.is_file())
            && let Ok(file) = File::open(path)
            && let Ok(selection) = archive::scan(&file, member)
        {
            return selection.finish(member);
        }

        let mut file = self.download(id, cached.as_deref().and_then(Path::parent))?;
        file.rewind().map_err(PullError::TempFile)?;
        let selection = archive::scan(file.as_file(), member)?;
        if let Some(path) = cached
            && file.path().parent() == path.parent()
        {
            let _ = file.persist(path);
        }
        selection.finish(member)
    }

    fn cache_path(&self, id: &ArxivId) -> Option<PathBuf> {
        if !id.versioned() {
            return None;
        }
        self.cache
            .as_ref()
            .map(|dir| dir.join(format!("{}.gz", id.0)))
    }

    fn download(&self, id: &ArxivId, cache_dir: Option<&Path>) -> Result<NamedTempFile> {
        let mut response = self
            .agent
            .get(format!("{}/src/{}", self.base_url, id.0))
            .header("Accept-Encoding", "identity")
            .call()
            .map_err(|err| PullError::Download(err.into()))?;
        match response.status().as_u16() {
            200 => {}
            404 | 410 => return Err(PullError::NoSource(id.0.clone())),
            status => return Err(PullError::Http(status)),
        }
        if response
            .body()
            .content_length()
            .is_some_and(|len| len > DOWNLOAD_LIMIT)
        {
            return Err(download_limit());
        }

        let cached_file = cache_dir.and_then(|dir| {
            fs::create_dir_all(dir).ok()?;
            NamedTempFile::new_in(dir).ok()
        });
        let mut file = match cached_file {
            Some(file) => file,
            None => NamedTempFile::new().map_err(PullError::TempFile)?,
        };
        let mut body = response
            .body_mut()
            .with_config()
            .limit(DOWNLOAD_LIMIT)
            .reader();
        io::copy(&mut body, &mut file).map_err(|err| {
            if matches!(
                err.get_ref()
                    .and_then(|err| err.downcast_ref::<ureq::Error>()),
                Some(ureq::Error::BodyExceedsLimit(_))
            ) {
                download_limit()
            } else {
                PullError::Download(err.into())
            }
        })?;
        Ok(file)
    }
}

impl Default for Arxiv {
    fn default() -> Self {
        Self::new()
    }
}

fn download_limit() -> PullError {
    PullError::Limit("compressed source", "100 MiB")
}

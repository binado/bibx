use std::path::PathBuf;

use crate::identifier::Identifier;

pub type Result<T> = std::result::Result<T, Error>;

/// Each variant's `Display` leaves out its [source](std::error::Error::source), so print the
/// whole chain to show the cause.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("no .bib file found, pass -f")]
    NoBibFile,
    #[error("several .bib files found, pass -f to choose one:{}", list_paths(.0))]
    ManyBibFiles(Vec<PathBuf>),
    #[error("{} is not valid UTF-8", .0.display())]
    NotUtf8(PathBuf),
    #[error("{}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid BibTeX: {message}")]
    Parse { message: String },
    #[error("no entry matches `{0}`")]
    NotFound(Identifier),
    #[error("`{id}` matches several entries: {}", .keys.join(", "))]
    Ambiguous { id: Identifier, keys: Vec<String> },
    #[error("`{0}` matches several INSPIRE records; use a record id or an eprint")]
    AmbiguousInspire(Identifier),
    #[error("an entry with key `{0}` already exists; pass --key to choose another key")]
    DuplicateKey(String),
    #[error("invalid citation key `{0}`")]
    InvalidKey(String),
    #[error("network error")]
    Network(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("INSPIRE rate limit reached (15 requests per 5 s); retry in at least 5 s")]
    RateLimited,
    #[error("{0}")]
    Provider(String),
    #[error("cannot fetch the bare key `{0}`; use inspire:{0}")]
    UnfetchableKey(String),
    #[error(
        "expected an arXiv or INSPIRE URL: https://arxiv.org/abs/<id>, https://arxiv.org/pdf/<id> or https://inspirehep.net/literature/<record-id>"
    )]
    InvalidIdentifierUrl,
}

fn list_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|p| format!("\n  {}", p.display()))
        .collect()
}

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }
}

use std::path::PathBuf;

use crate::identifier::Identifier;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no .bib file found, pass -f")]
    NoBibFile,
    #[error("several .bib files found, pass -f to choose one:{}", list_paths(.0))]
    ManyBibFiles(Vec<PathBuf>),
    #[error("{} is not valid UTF-8", .0.display())]
    NotUtf8(PathBuf),
    #[error("{}: {source}", path.display())]
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
    #[error("an entry with key `{0}` already exists; pass --key to choose another key")]
    DuplicateKey(String),
    #[error("invalid citation key `{0}`")]
    InvalidKey(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("{0}")]
    Provider(String),
    #[error("cannot fetch the bare key `{0}`; use inspire:{0}")]
    UnfetchableKey(String),
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

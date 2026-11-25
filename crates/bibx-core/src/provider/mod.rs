//! Where entries are fetched from. The identifier says what the paper is; the [`Source`]
//! says where to ask.

#[cfg(feature = "inspire")]
pub mod inspire;
#[cfg(feature = "testing")]
pub mod testing;

use crate::error::{Error, Result};
use crate::identifier::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Inspire,
}

/// What a provider returned for an identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched {
    /// The citation key the provider chose.
    pub key: String,
    /// The entry exactly as received, when the source sends BibTeX.
    pub bibtex: Option<String>,
}

pub trait Provider {
    fn fetch(&self, id: &Identifier) -> Result<Fetched>;
}

pub fn provider_for(source: Source) -> Box<dyn Provider> {
    match source {
        #[cfg(feature = "inspire")]
        Source::Inspire => Box::new(inspire::Inspire::new()),
        #[cfg(not(feature = "inspire"))]
        Source::Inspire => Box::new(Disabled("inspire")),
    }
}

#[cfg(not(feature = "inspire"))]
struct Disabled(&'static str);

#[cfg(not(feature = "inspire"))]
impl Provider for Disabled {
    fn fetch(&self, _: &Identifier) -> Result<Fetched> {
        Err(Error::Provider(format!(
            "this build of bibx has no {} support",
            self.0
        )))
    }
}

/// Parse a provider's BibTeX body, which must hold exactly one entry, and return its text
/// (from `@` to the closing delimiter) and key.
pub fn single_entry(body: &str) -> Result<Fetched> {
    let bib = bibx_parser::parse(body);
    if let Some((_, err)) = bib.errors().next() {
        return Err(Error::Parse {
            message: err.to_string(),
        });
    }
    let mut entries = bib.entries();
    match (entries.next(), entries.next()) {
        (Some(entry), None) => Ok(Fetched {
            key: entry.key.value.to_string(),
            bibtex: Some(entry.span.text(body).to_string()),
        }),
        (None, _) => Err(Error::Parse {
            message: "the response holds no BibTeX entry".into(),
        }),
        _ => Err(Error::Parse {
            message: "the response holds several BibTeX entries".into(),
        }),
    }
}

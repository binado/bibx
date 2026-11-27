//! Where entries are fetched from. The identifier says what the paper is; the [`Source`]
//! says where to ask.

#[cfg(feature = "inspire")]
pub mod inspire;
#[cfg(feature = "testing")]
pub mod testing;

use std::ops::Range;

use crate::error::{Error, Result};
use crate::identifier::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Source {
    Inspire,
}

/// What a provider returned for an identifier: one well-formed BibTeX entry. Only
/// [`single_entry`] builds one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched {
    /// From `@` to the closing delimiter.
    entry: String,
    /// The byte range of the citation key within `entry`.
    key: Range<usize>,
}

impl Fetched {
    /// The entry exactly as received, from `@` to the closing delimiter.
    pub fn entry(&self) -> &str {
        &self.entry
    }

    /// The citation key the provider chose.
    pub fn key(&self) -> &str {
        &self.entry[self.key.clone()]
    }

    /// The eprint and DOI the entry carries, as identifiers.
    pub(crate) fn identifiers(&self) -> Vec<Identifier> {
        let bib = bibx_parser::parse(&self.entry);
        let Some(entry) = bib.entries().next() else {
            return Vec::new();
        };
        [("eprint", "eprint:"), ("doi", "doi:")]
            .into_iter()
            .filter_map(|(field, prefix)| {
                let text = entry.field(field)?.text(&self.entry);
                Some(Identifier::parse(&format!("{prefix}{text}")))
            })
            .filter(|id| !id.value().is_empty())
            .collect()
    }

    /// The entry with only its citation key replaced by `key`.
    pub(crate) fn with_key(&self, key: &str) -> String {
        let mut entry = self.entry.clone();
        entry.replace_range(self.key.clone(), key);
        entry
    }
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

/// Parse a provider's BibTeX body, which must hold exactly one entry. Text around the entry
/// is dropped.
pub fn single_entry(body: &str) -> Result<Fetched> {
    let bib = bibx_parser::parse(body);
    if let Some((_, err)) = bib.errors().next() {
        return Err(Error::Parse {
            message: err.to_string(),
        });
    }
    let mut entries = bib.entries();
    match (entries.next(), entries.next()) {
        (Some(entry), None) => {
            let (start, key) = (entry.span.start, entry.key.span);
            Ok(Fetched {
                entry: entry.span.text(body).to_string(),
                key: key.start - start..key.end - start,
            })
        }
        (None, _) => Err(Error::Parse {
            message: "the response holds no BibTeX entry".into(),
        }),
        _ => Err(Error::Parse {
            message: "the response holds several BibTeX entries".into(),
        }),
    }
}

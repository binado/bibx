use std::time::Duration;

use crate::error::{Error, Result};
use crate::identifier::{Identifier, Kind, normalize_eprint};
use crate::provider::{Fetched, Provider, single_entry};

const DEFAULT_URL: &str = "https://inspirehep.net";
const URL_ENV: &str = "BIBX_INSPIRE_URL";

pub struct Inspire {
    base_url: String,
    agent: ureq::Agent,
}

impl Inspire {
    /// Talks to inspirehep.net, or to `$BIBX_INSPIRE_URL` when set (for tests).
    pub fn new() -> Self {
        let url = std::env::var(URL_ENV)
            .ok()
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| DEFAULT_URL.to_string());
        Self::with_base_url(url)
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .user_agent(concat!("bibx/", env!("CARGO_PKG_VERSION")))
            .http_status_as_error(false)
            .build()
            .into();
        Inspire {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            agent,
        }
    }

    /// The request path and query for `id`, and whether the answer is a search that must
    /// have exactly one hit.
    fn request(id: &Identifier) -> Result<(String, bool)> {
        let value = id.value();
        Ok(match id.kind() {
            Kind::Eprint => (
                format!(
                    "/api/arxiv/{}?format=bibtex",
                    encode_path(&normalize_eprint(value))
                ),
                false,
            ),
            Kind::Doi => (
                format!("/api/doi/{}?format=bibtex", encode_path(value)),
                false,
            ),
            Kind::Inspire if id.is_record_id() => {
                (format!("/api/literature/{value}?format=bibtex"), false)
            }
            Kind::Inspire => (
                format!(
                    "/api/literature?q=texkey%3A{}&size=2&format=bibtex",
                    encode_segment(value)
                ),
                true,
            ),
            Kind::Key | Kind::Ads => {
                return Err(Error::Provider(format!(
                    "INSPIRE cannot look up `{}` identifiers; use eprint:, doi: or inspire:",
                    id.kind().prefix()
                )));
            }
        })
    }
}

impl Default for Inspire {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for Inspire {
    fn fetch(&self, id: &Identifier) -> Result<Fetched> {
        let (target, is_search) = Self::request(id)?;
        let mut response = self
            .agent
            .get(format!("{}{target}", self.base_url))
            .call()
            .map_err(|e| Error::Network(e.into()))?;
        let status = response.status().as_u16();
        match status {
            200 => {}
            404 => return Err(Error::NotFound(id.clone())),
            429 => return Err(Error::RateLimited),
            other => {
                return Err(Error::Provider(format!(
                    "INSPIRE answered with HTTP {other}"
                )));
            }
        }
        let body = response
            .body_mut()
            .read_to_string()
            .map_err(|e| Error::Network(e.into()))?;
        if is_search {
            let hits = bibx_parser::parse(&body).entries().count();
            match hits {
                0 => return Err(Error::NotFound(id.clone())),
                1 => {}
                _ => return Err(Error::AmbiguousInspire(id.clone())),
            }
        }
        single_entry(&body)
    }
}

fn encode_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Percent-encode each `/`-separated segment, keeping the slashes (old arXiv ids and DOIs
/// contain them and the API takes them as path separators).
fn encode_path(s: &str) -> String {
    s.split('/')
        .map(encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(id: &str) -> String {
        Inspire::request(&Identifier::parse(id)).unwrap().0
    }

    #[test]
    fn builds_the_documented_requests() {
        assert_eq!(
            request("eprint:hep-th/9711200v2"),
            "/api/arxiv/hep-th/9711200?format=bibtex"
        );
        assert_eq!(
            request("eprint:2005.12345"),
            "/api/arxiv/2005.12345?format=bibtex"
        );
        assert_eq!(
            request("inspire:451647"),
            "/api/literature/451647?format=bibtex"
        );
        assert_eq!(
            request("doi:10.1103/PhysRevD.102.123456"),
            "/api/doi/10.1103/PhysRevD.102.123456?format=bibtex"
        );
        assert_eq!(
            request("inspire:Maldacena:1997re"),
            "/api/literature?q=texkey%3AMaldacena%3A1997re&size=2&format=bibtex"
        );
    }

    #[test]
    fn path_segments_are_percent_encoded() {
        assert_eq!(
            request("doi:10.1002/(SICI)1097 <x>"),
            "/api/doi/10.1002/%28SICI%291097%20%3Cx%3E?format=bibtex"
        );
    }

    #[test]
    fn keys_and_ads_ids_are_not_fetchable_here() {
        assert!(Inspire::request(&Identifier::parse("ads:2020ApJ...900..123A")).is_err());
        assert!(Inspire::request(&Identifier::parse("plain")).is_err());
    }
}

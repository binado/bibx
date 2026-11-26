use std::fmt;

use url::Url;

use crate::error::{self, Error};

/// What an identifier refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Key,
    Eprint,
    Doi,
    Inspire,
    Ads,
}

impl Kind {
    pub fn prefix(self) -> &'static str {
        match self {
            Kind::Key => "key",
            Kind::Eprint => "eprint",
            Kind::Doi => "doi",
            Kind::Inspire => "inspire",
            Kind::Ads => "ads",
        }
    }

    fn from_prefix(prefix: &str) -> Option<Kind> {
        if prefix.eq_ignore_ascii_case("arxiv") {
            return Some(Kind::Eprint);
        }
        [Kind::Key, Kind::Eprint, Kind::Doi, Kind::Inspire, Kind::Ads]
            .into_iter()
            .find(|k| k.prefix() == prefix)
    }
}

/// `<kind>:<value>`. A string without a known prefix is a bare citation key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub kind: Kind,
    pub value: String,
}

impl Identifier {
    /// Split on the first colon; only a known prefix counts as a kind, so `Author:2020abc`
    /// is a key. `key:` spells a key out when it would otherwise start with a known prefix.
    /// The `arxiv:` prefix is a case-insensitive alias for `eprint:`.
    pub fn parse(s: &str) -> Identifier {
        if let Some((prefix, value)) = s.split_once(':')
            && !value.is_empty()
            && let Some(kind) = Kind::from_prefix(prefix)
        {
            let value = match kind {
                Kind::Eprint => normalize_eprint(value),
                _ => value.to_string(),
            };
            return Identifier { kind, value };
        }
        Identifier {
            kind: Kind::Key,
            value: s.to_string(),
        }
    }

    /// Resolve an arXiv abstract or PDF URL, or an INSPIRE literature URL.
    /// arXiv versions, URL queries and fragments are discarded.
    pub fn from_url(input: &str) -> error::Result<Self> {
        let invalid = Error::InvalidIdentifierUrl;
        let url = Url::parse(input).map_err(|_| Error::InvalidIdentifierUrl)?;
        let Some((scheme, rest)) = input.split_once("://") else {
            return Err(invalid);
        };
        let Some((authority, path)) = rest.split(['?', '#']).next().unwrap_or("").split_once('/')
        else {
            return Err(invalid);
        };
        let host = url.host_str().unwrap_or("");
        let raw_host = authority.split(':').next().unwrap_or("");
        if !matches!(url.scheme(), "http" | "https")
            || !scheme.eq_ignore_ascii_case(url.scheme())
            || !raw_host.eq_ignore_ascii_case(host)
            || authority.contains('@')
            || url.port().is_some()
            || url.path().strip_prefix('/') != Some(path)
        {
            return Err(invalid);
        }

        match host {
            "arxiv.org" => {
                let id = path
                    .strip_prefix("abs/")
                    .or_else(|| {
                        path.strip_prefix("pdf/")
                            .map(|id| id.strip_suffix(".pdf").unwrap_or(id))
                    })
                    .filter(|id| is_arxiv_id(id))
                    .ok_or(Error::InvalidIdentifierUrl)?;
                Ok(Self {
                    kind: Kind::Eprint,
                    value: normalize_eprint(id),
                })
            }
            "inspirehep.net" => {
                let id = path
                    .strip_prefix("literature/")
                    .filter(|id| is_ascii_decimal(id))
                    .ok_or(Error::InvalidIdentifierUrl)?;
                Ok(Self {
                    kind: Kind::Inspire,
                    value: id.to_string(),
                })
            }
            _ => Err(invalid),
        }
    }

    /// For `inspire:` values: all digits is a record id, anything else a texkey.
    pub fn is_record_id(&self) -> bool {
        self.kind == Kind::Inspire && is_ascii_decimal(&self.value)
    }
}

fn is_ascii_decimal(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn is_arxiv_id(id: &str) -> bool {
    let (archive, number) = match id.split_once('/') {
        Some((archive, number)) => (Some(archive), number),
        None => (None, id),
    };
    let number = match number.split_once('v') {
        Some((number, version)) if is_ascii_decimal(version) => number,
        Some(_) => return false,
        None => number,
    };
    match archive {
        Some(archive) => {
            archive
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphabetic)
                && archive
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.'))
                && number.len() == 7
                && is_ascii_decimal(number)
        }
        None => number.split_once('.').is_some_and(|(date, serial)| {
            date.len() == 4
                && is_ascii_decimal(date)
                && matches!(serial.len(), 4 | 5)
                && is_ascii_decimal(serial)
        }),
    }
}

impl std::str::FromStr for Identifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Identifier::parse(s))
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            Kind::Key => f.write_str(&self.value),
            kind => write!(f, "{}:{}", kind.prefix(), self.value),
        }
    }
}

/// Strip an `arXiv:` prefix and a trailing version (`v2`) from an arXiv id.
pub fn normalize_eprint(s: &str) -> String {
    let s = s.trim();
    let s = match s.get(..6) {
        Some(p) if p.eq_ignore_ascii_case("arxiv:") => s.get(6..).unwrap_or(s),
        _ => s,
    };
    if let Some(v) = s.rfind('v') {
        let (head, tail) = s.split_at(v);
        let digits = &tail[1..];
        if !head.is_empty()
            && head.ends_with(|c: char| c.is_ascii_digit())
            && !digits.is_empty()
            && digits.bytes().all(|b| b.is_ascii_digit())
        {
            return head.to_string();
        }
    }
    s.to_string()
}

/// Lowercase a DOI and strip a resolver URL or `doi:` prefix.
pub fn normalize_doi(s: &str) -> String {
    let s = s.trim();
    for prefix in [
        "https://doi.org/",
        "http://doi.org/",
        "https://dx.doi.org/",
        "http://dx.doi.org/",
        "doi:",
    ] {
        if s.len() >= prefix.len()
            && let Some(head) = s.get(..prefix.len())
            && head.eq_ignore_ascii_case(prefix)
        {
            return s.get(prefix.len()..).unwrap_or("").to_ascii_lowercase();
        }
    }
    s.to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> (Kind, String) {
        let i = Identifier::parse(s);
        (i.kind, i.value)
    }

    #[test]
    fn bare_value_is_a_key() {
        assert_eq!(
            id("Maldacena:1997re"),
            (Kind::Key, "Maldacena:1997re".into())
        );
        assert_eq!(id("plain"), (Kind::Key, "plain".into()));
    }

    #[test]
    fn known_prefix_splits_on_the_first_colon_only() {
        assert_eq!(
            id("doi:10.1103/PhysRevD.102:12"),
            (Kind::Doi, "10.1103/PhysRevD.102:12".into())
        );
        assert_eq!(
            id("inspire:Author:2020abc"),
            (Kind::Inspire, "Author:2020abc".into())
        );
        assert_eq!(
            id("ads:2020ApJ...900..123A"),
            (Kind::Ads, "2020ApJ...900..123A".into())
        );
    }

    #[test]
    fn key_prefix_escapes_a_known_prefix() {
        assert_eq!(id("key:doi:weird"), (Kind::Key, "doi:weird".into()));
        assert_eq!(
            id("key:arxiv:2005.12345"),
            (Kind::Key, "arxiv:2005.12345".into())
        );
        assert_eq!(
            id("key:Author:2020abc"),
            (Kind::Key, "Author:2020abc".into())
        );
    }

    #[test]
    fn unknown_or_empty_prefix_is_a_whole_key() {
        assert_eq!(
            id("unknown:2005.12345"),
            (Kind::Key, "unknown:2005.12345".into())
        );
        assert_eq!(id("arxiv:"), (Kind::Key, "arxiv:".into()));
        assert_eq!(id("doi:"), (Kind::Key, "doi:".into()));
        assert_eq!(id(""), (Kind::Key, String::new()));
    }

    #[test]
    fn eprint_is_normalized() {
        assert_eq!(id("eprint:2005.12345v3").1, "2005.12345");
        assert_eq!(id("eprint:hep-th/9711200v2").1, "hep-th/9711200");
        assert_eq!(id("eprint:arXiv:2005.12345").1, "2005.12345");
        assert_eq!(id("arxiv:2005.12345v3"), id("eprint:2005.12345"));
        assert_eq!(id("arXiv:hep-th/9711200v2"), id("eprint:hep-th/9711200"));
        assert_eq!(normalize_eprint("hep-th/9711200"), "hep-th/9711200");
        assert_eq!(normalize_eprint("solv-int/9801001"), "solv-int/9801001");
    }

    #[test]
    fn publication_urls_resolve_to_existing_identifiers() {
        for (url, expected) in [
            ("https://arxiv.org/abs/2005.12345", "eprint:2005.12345"),
            ("https://arxiv.org/pdf/0704.0001v2", "eprint:0704.0001"),
            (
                "http://ARXIV.ORG:80/pdf/hep-th/9711200v2.pdf?download=1#page=2",
                "eprint:hep-th/9711200",
            ),
            (
                "https://arxiv.org/abs/solv-int/9801001",
                "eprint:solv-int/9801001",
            ),
            (
                "https://inspirehep.net/literature/451647?x=1#abstract",
                "inspire:451647",
            ),
        ] {
            assert_eq!(
                Identifier::from_url(url).unwrap(),
                Identifier::parse(expected),
                "{url}"
            );
        }
    }

    #[test]
    fn invalid_publication_urls_are_rejected() {
        for url in [
            "",
            "arxiv.org/abs/2005.12345",
            "https:arxiv.org/abs/2005.12345",
            "ftp://arxiv.org/abs/2005.12345",
            "https://arxiv.org.example.com/abs/2005.12345",
            "https://reader@arxiv.org/abs/2005.12345",
            "https://@arxiv.org/abs/2005.12345",
            "https://arxiv.org:444/abs/2005.12345",
            "https://arxiv.org/search/2005.12345",
            "https://arxiv.org/abs/2005.12345.pdf",
            "https://arxiv.org/abs/2005.12345/extra",
            "https://arxiv.org/extra/../abs/2005.12345",
            "https://arxiv.org/abs/2005.12345v",
            "https://arxiv.org/abs/2005.123",
            "https://arxiv.org/pdf/hep-th/971120.pdf",
            "https://inspirehep.net/literature/",
            "https://inspirehep.net/literature/Maldacena:1997re",
            "https://inspirehep.net/literature/451647/extra",
        ] {
            assert!(
                matches!(Identifier::from_url(url), Err(Error::InvalidIdentifierUrl)),
                "{url}"
            );
        }
    }

    #[test]
    fn doi_is_normalized() {
        assert_eq!(
            normalize_doi("https://doi.org/10.1103/PhysRevD.1"),
            "10.1103/physrevd.1"
        );
        assert_eq!(normalize_doi("DOI:10.1/AB"), "10.1/ab");
    }

    #[test]
    fn record_id_vs_texkey() {
        assert!(Identifier::parse("inspire:451647").is_record_id());
        assert!(!Identifier::parse("inspire:Maldacena:1997re").is_record_id());
        assert!(!Identifier::parse("451647").is_record_id());
    }

    #[test]
    fn display_round_trips() {
        for s in ["doi:10.1/x", "inspire:Foo:2020abc", "Foo:2020abc"] {
            assert_eq!(Identifier::parse(s).to_string(), s);
        }
        assert_eq!(
            Identifier::parse("arXiv:2005.12345v3").to_string(),
            "eprint:2005.12345"
        );
    }
}

use std::path::{Path, PathBuf};

use bibx_parser::{Bib, Entry};

use crate::error::{Error, Result};
use crate::identifier::{Identifier, Kind, normalize_doi, normalize_eprint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    Crlf,
}

impl LineEnding {
    pub fn as_str(self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }

    /// From the first newline; LF when there is none.
    pub fn detect(src: &str) -> LineEnding {
        match src.find('\n') {
            Some(i) if src.as_bytes().get(i.wrapping_sub(1)) == Some(&b'\r') => LineEnding::Crlf,
            _ => LineEnding::Lf,
        }
    }
}

/// A bib file's text, which is never re-serialized, only spliced.
#[derive(Debug, Clone)]
pub struct BibFile {
    pub path: PathBuf,
    pub src: String,
    pub line_ending: LineEnding,
}

impl BibFile {
    pub fn load(path: &Path) -> Result<BibFile> {
        let bytes = std::fs::read(path).map_err(|e| Error::io(path, e))?;
        let src = String::from_utf8(bytes).map_err(|_| Error::NotUtf8(path.to_path_buf()))?;
        Ok(BibFile::from_source(path, src))
    }

    pub fn from_source(path: impl Into<PathBuf>, src: String) -> BibFile {
        let line_ending = LineEnding::detect(&src);
        BibFile {
            path: path.into(),
            src,
            line_ending,
        }
    }

    pub fn parse(&self) -> Bib<'_> {
        bibx_parser::parse(&self.src)
    }

    /// The entries `id` refers to. A key matches exactly; an eprint or DOI matches the
    /// normalized `eprint` or `doi` field; `inspire:` and `ads:` match the key (both services
    /// use their own id as the key).
    pub fn find<'b, 's>(&self, bib: &'b Bib<'s>, id: &Identifier) -> Vec<&'b Entry<'s>> {
        let wanted_eprint = normalize_eprint(id.value());
        let wanted_doi = normalize_doi(id.value());
        bib.entries()
            .filter(|e| match id.kind() {
                Kind::Key | Kind::Inspire | Kind::Ads => e.key.value == id.value(),
                Kind::Eprint => e
                    .field("eprint")
                    .is_some_and(|f| normalize_eprint(f.text(&self.src)) == wanted_eprint),
                Kind::Doi => e
                    .field("doi")
                    .is_some_and(|f| normalize_doi(f.text(&self.src)) == wanted_doi),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "@article{A:1, eprint = \"hep-th/9711200\", doi = {10.4310/ATMP.1998}}\n@article{B:2, eprint={2005.12345v2}, doi=\"https://doi.org/10.1/X\"}\n";

    fn keys(src: &str, id: &str) -> Vec<String> {
        let f = BibFile::from_source("x.bib", src.to_string());
        let bib = f.parse();
        f.find(&bib, &Identifier::parse(id))
            .iter()
            .map(|e| e.key.value.to_string())
            .collect()
    }

    #[test]
    fn finds_by_key_eprint_and_doi() {
        assert_eq!(keys(SRC, "A:1"), ["A:1"]);
        assert_eq!(keys(SRC, "key:B:2"), ["B:2"]);
        assert_eq!(keys(SRC, "eprint:2005.12345"), ["B:2"]);
        assert_eq!(keys(SRC, "eprint:hep-th/9711200v1"), ["A:1"]);
        assert_eq!(keys(SRC, "doi:10.4310/atmp.1998"), ["A:1"]);
        assert_eq!(keys(SRC, "doi:10.1/x"), ["B:2"]);
        assert_eq!(keys(SRC, "inspire:A:1"), ["A:1"]);
        assert!(keys(SRC, "nope").is_empty());
    }

    #[test]
    fn detects_line_endings() {
        assert_eq!(LineEnding::detect("a\r\nb\n"), LineEnding::Crlf);
        assert_eq!(LineEnding::detect("a\nb\r\n"), LineEnding::Lf);
        assert_eq!(LineEnding::detect("no newline"), LineEnding::Lf);
        assert_eq!(LineEnding::detect("\nstarts"), LineEnding::Lf);
    }

    #[test]
    fn rejects_invalid_utf8() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a.bib");
        std::fs::write(&p, [0xff, 0xfe, b'@']).unwrap();
        assert!(matches!(BibFile::load(&p), Err(Error::NotUtf8(_))));
    }
}

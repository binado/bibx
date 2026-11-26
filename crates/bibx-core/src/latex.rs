//! LaTeX to Unicode, for showing and ordering values. Decoded text is never written back.

use biblatex::{Bibliography, ChunksExt, Pair, RawBibliography, RawChunk, RawEntry, Spanned};

/// biblatex picks the parse mode and resolves cross-references by field name, so the value
/// goes under a name that means nothing to it.
const FIELD: &str = "x";

/// The text of one braced or quoted value, given without its delimiters, or `None` if
/// biblatex rejects it.
///
/// Math, unknown commands and `~` are kept. Line breaks may remain in the result.
pub(crate) fn decode(latex: &str) -> Option<String> {
    // Built by hand so that the value cannot close a wrapper and add fields.
    let value = vec![Spanned::zero(RawChunk::Normal(latex))];
    let entry = RawEntry {
        key: Spanned::zero(""),
        kind: Spanned::zero("misc"),
        fields: vec![Pair::new(Spanned::zero(FIELD), Spanned::zero(value))],
    };
    let bibliography = Bibliography::from_raw(RawBibliography {
        preamble: String::new(),
        entries: vec![Spanned::zero(entry)],
        abbreviations: Vec::new(),
    })
    .ok()?;
    Some(bibliography.get("")?.get(FIELD)?.format_verbatim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes() {
        for (latex, expected) in [
            (r#"Schr{\"o}dinger"#, "Schrödinger"),
            (r"{\'{\i}}", "í"),
            (r"{The {LHC}} \& 5\%", "The LHC & 5%"),
            ("231--252", "231–252"),
            (r"{\textbackslash}{\&}", r"\&"),
        ] {
            assert_eq!(decode(latex).as_deref(), Some(expected), "{latex}");
        }
    }

    #[test]
    fn keeps_what_it_does_not_decode() {
        for latex in [r"$\alpha_s$", r"$$a$$", r"\foo{bar}", "A~B"] {
            assert_eq!(decode(latex).as_deref(), Some(latex), "{latex}");
        }
    }

    #[test]
    fn rejects_malformed_values() {
        for latex in ["Cost $5", "a}", r"a\", "a}, crossref = {x"] {
            assert_eq!(decode(latex), None, "{latex}");
        }
    }

    /// Known limits of biblatex, recorded so that an upgrade that changes them is noticed.
    #[test]
    fn upstream_quirks() {
        for (latex, actual) in [
            // A dotless i under an unbraced accent should give `í`.
            (r"{\'\i}", "\\\u{301}i"),
            // A space appears after an unknown command, even before punctuation.
            (r"\apjl", r"\apjl "),
            (r"\apjl, 5", r"\apjl , 5"),
        ] {
            assert_eq!(decode(latex).as_deref(), Some(actual), "{latex}");
        }
    }
}

use std::borrow::Cow;

use bibx_parser::{Bib, Field, Spanned, ValuePart};
use indexmap::IndexMap;

use crate::latex;

/// Fields that hold identifiers, URLs, paths or citation keys rather than LaTeX. biblatex
/// still strips braces and rewrites `--` in its verbatim mode, so they are not decoded at
/// all. The first group copies its private list of verbatim fields.
const OPAQUE_FIELDS: &[&str] = &[
    "file", "doi", "uri", "eprint", "verba", "verbb", "verbc", "pdf", "url", "urlraw",
    // Citation keys.
    "crossref", "xref", "xdata", "entryset", "ids", // The ADS record link.
    "adsurl",
];

/// A field value, with whitespace runs collapsed to one space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value(Repr);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Repr {
    /// The inside of a single braced or quoted value.
    Latex(String),
    /// Source text that is not decoded.
    Literal(String),
}

impl Value {
    fn new(src: &str, field: &Field<'_>) -> Value {
        let opaque = OPAQUE_FIELDS
            .iter()
            .any(|name| field.name.value.eq_ignore_ascii_case(name));
        Value(match field.value.as_slice() {
            [
                Spanned {
                    value: ValuePart::Braced(inner) | ValuePart::Quoted(inner),
                    ..
                },
            ] => {
                let text = collapse_ws(inner);
                if opaque {
                    Repr::Literal(text)
                } else {
                    Repr::Latex(text)
                }
            }
            _ => Repr::Literal(collapse_ws(field.raw(src))),
        })
    }

    /// The source text: a single braced or quoted value without its delimiters, and
    /// anything else (macros, `#` joins, bare numbers) as written.
    pub fn raw(&self) -> &str {
        match &self.0 {
            Repr::Latex(text) | Repr::Literal(text) => text,
        }
    }

    /// The text to show and order by: LaTeX turned into Unicode where possible.
    ///
    /// Math, unknown commands and `~` are kept. This is [`raw`](Value::raw) for macros,
    /// `#` joins, bare numbers, identifier, URL and citation-key fields, and values that
    /// cannot be decoded. Each call decodes again.
    pub fn decoded(&self) -> Cow<'_, str> {
        if let Repr::Latex(text) = &self.0
            && let Some(decoded) = latex::decode(text)
        {
            return Cow::Owned(collapse_ws(&decoded));
        }
        Cow::Borrowed(self.raw())
    }
}

/// One entry, as shown by `list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub key: String,
    pub entry_type: String,
    /// The requested fields that the entry has, by lowercased name.
    pub fields: IndexMap<String, Value>,
}

/// What to sort by. `key` and `type` are the pseudo-fields; any other name is a bib field,
/// ordered by its [decoded](Value::decoded) value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortKey {
    Key,
    Type,
    Field(String),
}

impl SortKey {
    pub fn parse(name: &str) -> SortKey {
        match name.to_ascii_lowercase().as_str() {
            "key" => SortKey::Key,
            "type" => SortKey::Type,
            other => SortKey::Field(other.to_string()),
        }
    }
}

/// A sort value: missing last, numbers before text, text case-insensitive.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Sortable {
    Number(i64),
    Text(String),
    Missing,
}

impl Sortable {
    fn from_text(text: Option<String>) -> Sortable {
        match text {
            None => Sortable::Missing,
            Some(t) => match t.trim().parse() {
                Ok(n) => Sortable::Number(n),
                Err(_) => Sortable::Text(t.to_lowercase()),
            },
        }
    }
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The entries of `bib` as rows carrying `fields` (names other than the pseudo-fields `key`
/// and `type`, which every row has anyway). The sort is stable.
pub fn list(src: &str, bib: &Bib, sort: Option<&SortKey>, fields: &[String]) -> Vec<Row> {
    let mut rows: Vec<(Sortable, Row)> = bib
        .entries()
        .map(|entry| {
            let value = |name: &str| entry.field(name).map(|f| Value::new(src, f));
            let mut row_fields = IndexMap::new();
            for name in fields {
                let name = name.to_ascii_lowercase();
                if matches!(name.as_str(), "key" | "type") {
                    continue;
                }
                if let Some(v) = value(&name) {
                    row_fields.insert(name, v);
                }
            }
            let sortable = match sort {
                None => Sortable::Missing,
                Some(SortKey::Key) => Sortable::from_text(Some(entry.key.value.to_string())),
                Some(SortKey::Type) => Sortable::from_text(Some(entry.kind.value.to_string())),
                Some(SortKey::Field(name)) => {
                    Sortable::from_text(value(name).map(|v| v.decoded().into_owned()))
                }
            };
            (
                sortable,
                Row {
                    key: entry.key.value.to_string(),
                    entry_type: entry.kind.value.to_ascii_lowercase(),
                    fields: row_fields,
                },
            )
        })
        .collect();
    if sort.is_some() {
        rows.sort_by(|a, b| a.0.cmp(&b.0));
    }
    rows.into_iter().map(|(_, row)| row).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "@article{b, year = 2001, title = {Beta\n   two}}\n@book{a, year = {1999}}\n@misc{c, title = {No year}}\n@misc{d, year = 1999, title = {Zed}}\n@misc{e, year = {n.d.}}\n";

    fn keys(sort: Option<&str>) -> Vec<String> {
        let bib = bibx_parser::parse(SRC);
        let key = sort.map(SortKey::parse);
        list(SRC, &bib, key.as_ref(), &[])
            .into_iter()
            .map(|r| r.key)
            .collect()
    }

    #[test]
    fn source_order_without_sort() {
        assert_eq!(keys(None), ["b", "a", "c", "d", "e"]);
    }

    #[test]
    fn year_sorts_numerically_stable_with_missing_last() {
        // numbers, then text, then missing; ties keep source order (a before d)
        assert_eq!(keys(Some("year")), ["a", "d", "b", "e", "c"]);
    }

    #[test]
    fn sorts_by_key_and_type() {
        assert_eq!(keys(Some("key")), ["a", "b", "c", "d", "e"]);
        assert_eq!(keys(Some("type")), ["b", "a", "c", "d", "e"]);
    }

    #[test]
    fn rows_carry_requested_fields_only_when_present() {
        let bib = bibx_parser::parse(SRC);
        let rows = list(
            SRC,
            &bib,
            None,
            &["Title".into(), "year".into(), "key".into()],
        );
        assert_eq!(rows[0].fields["title"].raw(), "Beta two");
        assert_eq!(rows[0].fields["year"].raw(), "2001");
        assert_eq!(rows[1].fields.len(), 1);
        assert_eq!(rows[0].entry_type, "article");
        assert!(!rows[0].fields.contains_key("key"));
    }

    /// The only field of a one-entry bibliography.
    fn field(name: &str, source: &str) -> Value {
        let src = format!("@misc{{k, {name} = {source}}}");
        let bib = bibx_parser::parse(&src);
        let mut rows = list(&src, &bib, None, &[name.into()]);
        rows.remove(0).fields.pop().expect("one field").1
    }

    #[test]
    fn braced_and_quoted_values_are_decoded_but_keep_their_source() {
        for source in [r#"{Schr{\"o}dinger}"#, r#""Schr{\"o}dinger""#] {
            let value = field("title", source);
            assert_eq!(value.raw(), r#"Schr{\"o}dinger"#);
            assert_eq!(value.decoded(), "Schrödinger");
        }
    }

    #[test]
    fn decoded_whitespace_is_collapsed() {
        assert_eq!(field("title", "{a\\\nb}").decoded(), r"a\ b");
        assert_eq!(field("journal", r"{\apjl}").decoded(), r"\apjl");
    }

    #[test]
    fn opaque_fields_are_not_decoded() {
        for name in OPAQUE_FIELDS.iter().copied().chain(["DOI", "AdsUrl"]) {
            let value = field(name, "{a--b_{c}}");
            assert_eq!(value.decoded(), "a--b_{c}", "{name}");
        }
    }

    #[test]
    fn cross_references_are_left_alone() {
        for (name, source, expected) in [
            ("crossref", "{}", ""),
            ("crossref", "{k}", "k"),
            ("xdata", "{k}", "k"),
        ] {
            assert_eq!(field(name, source).decoded(), expected, "{name}");
        }
    }

    #[test]
    fn macros_joins_numbers_and_malformed_values_stay_raw() {
        for (source, expected) in [
            ("jan", "jan"),
            ("{a--b} # foo", "{a--b} # foo"),
            ("1999", "1999"),
            ("{Cost $5 -- {more}}", "Cost $5 -- {more}"),
        ] {
            assert_eq!(field("title", source).decoded(), expected, "{source}");
        }
    }

    fn keys_sorted_by(src: &str, name: &str) -> Vec<String> {
        let bib = bibx_parser::parse(src);
        list(src, &bib, Some(&SortKey::parse(name)), &[])
            .into_iter()
            .map(|r| r.key)
            .collect()
    }

    #[test]
    fn sorts_by_decoded_text() {
        let src = concat!(
            "@misc{zed, title = {Zed}}\n",
            "@misc{braced, title = {{Beta}}}\n",
            "@misc{plain, title = {Alpha}}\n",
            "@misc{broken, title = {Cost $5}}\n",
            "@misc{tie, title = {B{e}ta}}\n",
        );
        assert_eq!(
            keys_sorted_by(src, "title"),
            ["plain", "braced", "tie", "broken", "zed"]
        );
    }

    #[test]
    fn numbers_are_parsed_after_decoding() {
        let src = concat!(
            "@misc{range, pages = {10--12}}\n",
            "@misc{ten, pages = {10}}\n",
            "@misc{nine, pages = {{9}}}\n",
        );
        assert_eq!(keys_sorted_by(src, "pages"), ["nine", "ten", "range"]);
    }
}

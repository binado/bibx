use std::cmp::Ordering;

use bibx_parser::Bib;
use indexmap::IndexMap;

/// One entry, as shown by `list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub key: String,
    pub entry_type: String,
    /// The requested fields that the entry has, by lowercased name, with whitespace runs
    /// collapsed to one space.
    pub fields: IndexMap<String, String>,
}

/// What to sort by. `key` and `type` are the pseudo-fields; any other name is a bib field.
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
#[derive(Debug, PartialEq, Eq)]
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

impl Ord for Sortable {
    fn cmp(&self, other: &Self) -> Ordering {
        use Sortable::*;
        match (self, other) {
            (Number(a), Number(b)) => a.cmp(b),
            (Text(a), Text(b)) => a.cmp(b),
            (Number(_), _) => Ordering::Less,
            (_, Number(_)) => Ordering::Greater,
            (Text(_), Missing) => Ordering::Less,
            (Missing, Text(_)) => Ordering::Greater,
            (Missing, Missing) => Ordering::Equal,
        }
    }
}

impl PartialOrd for Sortable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
            let value = |name: &str| entry.field(name).map(|f| collapse_ws(f.text(src)));
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
                Some(SortKey::Field(name)) => Sortable::from_text(value(name)),
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
        assert_eq!(rows[0].fields["title"], "Beta two");
        assert_eq!(rows[0].fields["year"], "2001");
        assert_eq!(rows[1].fields.len(), 1);
        assert_eq!(rows[0].entry_type, "article");
        assert!(!rows[0].fields.contains_key("key"));
    }
}

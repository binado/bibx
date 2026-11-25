//! For every fixture `biblatex` accepts, both parsers must find the same entries with the
//! same type, key, field names and raw values.

mod common;

use biblatex::{RawBibliography, RawChunk};
use bibx_parser::{Item, ValuePart};

#[derive(Debug, PartialEq)]
struct Flat {
    kind: String,
    key: String,
    fields: Vec<(String, Vec<String>)>,
}

fn ours(src: &str) -> Vec<Flat> {
    bibx_parser::parse(src)
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Entry(e) => Some(Flat {
                kind: e.kind.value.to_ascii_lowercase(),
                key: e.key.value.to_string(),
                fields: e
                    .fields
                    .iter()
                    .map(|f| {
                        let parts = f
                            .value
                            .iter()
                            .map(|p| match p.value {
                                ValuePart::Braced(t)
                                | ValuePart::Quoted(t)
                                | ValuePart::Number(t) => t.to_string(),
                                ValuePart::Macro(t) => format!("${t}"),
                            })
                            .collect();
                        (f.name.value.to_ascii_lowercase(), parts)
                    })
                    .collect(),
            }),
            _ => None,
        })
        .collect()
}

fn theirs(raw: &RawBibliography) -> Vec<Flat> {
    raw.entries
        .iter()
        .map(|e| Flat {
            kind: e.v.kind.v.to_ascii_lowercase(),
            key: e.v.key.v.to_string(),
            fields: e
                .v
                .fields
                .iter()
                .map(|p| {
                    let parts = p
                        .value
                        .v
                        .iter()
                        .map(|c| match &c.v {
                            RawChunk::Normal(t) => t.to_string(),
                            // Their macro values include trailing whitespace.
                            RawChunk::Abbreviation(t) => format!("${}", t.trim_end()),
                        })
                        .collect();
                    (p.key.v.to_ascii_lowercase(), parts)
                })
                .collect(),
        })
        .collect()
}

#[test]
fn matches_biblatex_on_accepted_fixtures() {
    let mut compared = 0;
    for path in common::fixtures() {
        let src = common::read(&path);
        let Ok(raw) = RawBibliography::parse(&src) else {
            continue;
        };
        assert_eq!(ours(&src), theirs(&raw), "{}", path.display());
        compared += 1;
    }
    assert!(compared >= 15, "only {compared} fixtures were compared");
}

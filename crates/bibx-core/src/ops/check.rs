use std::collections::HashMap;
use std::collections::hash_map::Entry;

use bibx_parser::Item;

use crate::BibFile;
use crate::identifier::{Identifier, normalize_doi, normalize_eprint};

/// One-based source coordinates. A column counts Unicode characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Diagnostic<'s> {
    pub location: Location,
    pub kind: DiagnosticKind<'s>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DiagnosticKind<'s> {
    Syntax(String),
    DuplicateKey {
        key: &'s str,
        first: Location,
    },
    /// An entry's key differs from an earlier key only in case, which BibTeX treats as a clash.
    KeyCaseConflict {
        key: &'s str,
        first_key: &'s str,
        first: Location,
    },
    /// An entry repeats the eprint or DOI of an earlier entry under another key.
    DuplicatePaper {
        id: Identifier,
        first_key: &'s str,
        first: Location,
    },
}

/// Report recovered syntax errors and repeated citation keys (case-insensitive), eprints and DOIs
/// in source order.
pub fn check(file: &BibFile) -> Vec<Diagnostic<'_>> {
    let line_starts: Vec<_> = std::iter::once(0)
        .chain(file.src.match_indices('\n').map(|(offset, _)| offset + 1))
        .collect();
    let location = |offset| {
        let line = line_starts.partition_point(|&start| start <= offset);
        let start = line_starts[line - 1];
        Location {
            line,
            column: file.src[start..offset].chars().count() + 1,
        }
    };

    let mut diagnostics = Vec::new();
    let mut keys: HashMap<String, (&str, usize)> = HashMap::new();
    let mut papers: HashMap<Identifier, (&str, usize)> = HashMap::new();
    for item in file.parse().items {
        match item {
            Item::Error { error, .. } => diagnostics.push(Diagnostic {
                location: location(error.offset),
                kind: DiagnosticKind::Syntax(error.message),
            }),
            Item::Entry(entry) if !entry.key.value.is_empty() => {
                let key = entry.key.value;
                let offset = entry.key.span.start;
                let ids = [
                    ("eprint", "eprint:", normalize_eprint as fn(&str) -> String),
                    ("doi", "doi:", normalize_doi),
                ]
                .into_iter()
                .filter_map(|(field, prefix, normalize)| {
                    let text = normalize(entry.field(field)?.text(&file.src));
                    (!text.is_empty()).then(|| Identifier::parse(&format!("{prefix}{text}")))
                });
                for id in ids {
                    match papers.entry(id.clone()) {
                        Entry::Vacant(first) => {
                            first.insert((key, offset));
                        }
                        Entry::Occupied(first) if first.get().0 != key => {
                            let (first_key, first_offset) = *first.get();
                            diagnostics.push(Diagnostic {
                                location: location(offset),
                                kind: DiagnosticKind::DuplicatePaper {
                                    id,
                                    first_key,
                                    first: location(first_offset),
                                },
                            });
                        }
                        Entry::Occupied(_) => {}
                    }
                }
                match keys.entry(key.to_lowercase()) {
                    Entry::Vacant(first) => {
                        first.insert((key, offset));
                    }
                    Entry::Occupied(first) => {
                        let (first_key, first_offset) = *first.get();
                        let first = location(first_offset);
                        let location = location(offset);
                        diagnostics.push(Diagnostic {
                            location,
                            kind: if first_key == key {
                                DiagnosticKind::DuplicateKey { key, first }
                            } else {
                                DiagnosticKind::KeyCaseConflict {
                                    key,
                                    first_key,
                                    first,
                                }
                            },
                        });
                    }
                }
            }
            _ => {}
        }
    }
    diagnostics.sort_by_key(|diagnostic| diagnostic.location);
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locates_unicode_keys_on_a_crlf_line_and_errors_at_eof() {
        let file = BibFile::from_source(
            "test.bib",
            "% comment\r\n@a{é} @b{é}\r\n@a{k, title = {unfinished".into(),
        );
        let diagnostics = check(&file);
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(
            diagnostics[0],
            Diagnostic {
                location: Location {
                    line: 2,
                    column: 10,
                },
                kind: DiagnosticKind::DuplicateKey {
                    key: "é",
                    first: Location { line: 2, column: 4 },
                },
            }
        );
        assert_eq!(
            diagnostics[1].location,
            Location {
                line: 3,
                column: 26,
            }
        );
        assert!(matches!(diagnostics[1].kind, DiagnosticKind::Syntax(_)));
    }
}

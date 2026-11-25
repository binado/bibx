use crate::bibfile::BibFile;
use crate::error::{Error, Result};
use crate::identifier::Identifier;

/// The result of `remove`: the whole new file text and what was taken out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovePlan {
    pub new_src: String,
    /// The entry itself, from `@` to its closing delimiter.
    pub removed_text: String,
    pub key: String,
}

/// Delete the one entry `id` refers to, and only the text that belongs to it: its lines,
/// plus one blank line so no double blank line is left behind.
pub fn remove(file: &BibFile, id: &Identifier) -> Result<RemovePlan> {
    let bib = file.parse();
    let matches = file.find(&bib, id);
    let entry = match matches.as_slice() {
        [] => return Err(Error::NotFound(id.clone())),
        [entry] => entry,
        many => {
            return Err(Error::Ambiguous {
                id: id.clone(),
                keys: many.iter().map(|e| e.key.value.to_string()).collect(),
            });
        }
    };
    let src = file.src.as_str();
    let (start, end) = removal_range(src, entry.span.start, entry.span.end);
    let (head, tail) = (src.get(..start), src.get(end..));
    let new_src = match (head, tail) {
        (Some(head), Some(tail)) => format!("{head}{tail}"),
        _ => {
            return Err(Error::Parse {
                message: "entry range is out of bounds".into(),
            });
        }
    };
    Ok(RemovePlan {
        new_src,
        removed_text: entry.span.text(src).to_string(),
        key: entry.key.value.to_string(),
    })
}

fn is_blank(src: &str, start: usize, end: usize) -> bool {
    src.get(start..end).is_some_and(|s| s.trim().is_empty())
}

fn line_start(src: &str, at: usize) -> usize {
    src.get(..at)
        .and_then(|s| s.rfind('\n'))
        .map_or(0, |i| i + 1)
}

/// The end of the line holding `at`, after its newline (or the end of the text).
fn line_end(src: &str, at: usize) -> usize {
    src.get(at..)
        .and_then(|s| s.find('\n'))
        .map_or(src.len(), |i| at + i + 1)
}

/// The byte range to delete for an entry spanning `start..end`.
fn removal_range(src: &str, start: usize, end: usize) -> (usize, usize) {
    let first = line_start(src, start);
    let last = line_end(src, end);
    // Something else shares a line with the entry: take just the entry.
    if !is_blank(src, first, start) || !is_blank(src, end, last) {
        return (start, end);
    }
    let (mut from, mut to) = (first, last);
    let prev_blank = from > 0 && {
        let prev = line_start(src, from - 1);
        is_blank(src, prev, from)
    };
    let next_blank = to < src.len() && is_blank(src, to, line_end(src, to));
    if next_blank && (prev_blank || from == 0) {
        to = line_end(src, to);
    } else if to >= src.len() && prev_blank {
        from = line_start(src, from - 1);
    }
    (from, to)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rm(src: &str, id: &str) -> Result<RemovePlan> {
        remove(
            &BibFile::from_source("x.bib", src.to_string()),
            &Identifier::parse(id),
        )
    }

    fn after(src: &str, id: &str) -> String {
        rm(src, id).unwrap().new_src
    }

    #[test]
    fn first_middle_and_last_entries_leave_single_blank_lines() {
        let src = "@a{a}\n\n@b{b}\n\n@c{c}\n";
        assert_eq!(after(src, "a"), "@b{b}\n\n@c{c}\n");
        assert_eq!(after(src, "b"), "@a{a}\n\n@c{c}\n");
        assert_eq!(after(src, "c"), "@a{a}\n\n@b{b}\n");
    }

    #[test]
    fn tightly_packed_entries() {
        let src = "@a{a}\n@b{b}\n@c{c}\n";
        assert_eq!(after(src, "b"), "@a{a}\n@c{c}\n");
        assert_eq!(after(src, "a"), "@b{b}\n@c{c}\n");
        assert_eq!(after(src, "c"), "@a{a}\n@b{b}\n");
    }

    #[test]
    fn one_blank_line_side_is_kept() {
        assert_eq!(after("@a{a}\n@b{b}\n\n@c{c}\n", "b"), "@a{a}\n\n@c{c}\n");
        assert_eq!(after("@a{a}\n\n@b{b}\n@c{c}\n", "b"), "@a{a}\n\n@c{c}\n");
    }

    #[test]
    fn multi_line_entry_without_a_final_newline() {
        let src = "@a{a}\n\n@b{b,\n  title = {x}\n}";
        assert_eq!(after(src, "b"), "@a{a}\n");
        assert_eq!(after("@only{k}", "k"), "");
        assert_eq!(after("@only{k}\n", "k"), "");
    }

    #[test]
    fn indented_entry_loses_its_indentation_and_comments_stay() {
        let src = "% keep\n  @a{a,\n    x = 1\n  }  \n@b{b}\n";
        assert_eq!(after(src, "a"), "% keep\n@b{b}\n");
    }

    #[test]
    fn entries_sharing_a_line_take_only_the_entry() {
        assert_eq!(after("@a{a} @b{b}\n", "a"), " @b{b}\n");
    }

    #[test]
    fn crlf_is_preserved() {
        let src = "@a{a}\r\n\r\n@b{b}\r\n\r\n@c{c}\r\n";
        assert_eq!(after(src, "b"), "@a{a}\r\n\r\n@c{c}\r\n");
        assert_eq!(after(src, "c"), "@a{a}\r\n\r\n@b{b}\r\n");
    }

    #[test]
    fn reports_the_removed_entry_and_key() {
        let plan = rm("@a{k1, t = {x}}\n", "k1").unwrap();
        assert_eq!(plan.removed_text, "@a{k1, t = {x}}");
        assert_eq!(plan.key, "k1");
    }

    #[test]
    fn matches_by_eprint_and_doi() {
        let src = "@a{k1, eprint = \"2005.12345v2\"}\n@a{k2, doi = {10.1/X}}\n";
        assert_eq!(after(src, "eprint:2005.12345"), "@a{k2, doi = {10.1/X}}\n");
        assert_eq!(
            after(src, "doi:10.1/x"),
            "@a{k1, eprint = \"2005.12345v2\"}\n"
        );
    }

    #[test]
    fn not_found_and_ambiguous() {
        assert!(matches!(rm("@a{k}\n", "nope"), Err(Error::NotFound(_))));
        let err = rm(
            "@a{k, eprint = {1.1}}\n@a{j, eprint = {1.1v2}}\n",
            "eprint:1.1",
        )
        .unwrap_err();
        assert!(matches!(err, Error::Ambiguous { keys, .. } if keys == ["k", "j"]));
    }
}

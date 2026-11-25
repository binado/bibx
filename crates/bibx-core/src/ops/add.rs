use crate::bibfile::BibFile;
use crate::error::{Error, Result};
use crate::provider::{Fetched, single_entry};

/// The result of `add`: the whole new file text and the entry that was appended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddPlan {
    pub new_src: String,
    pub key: String,
    /// The appended entry, with the file's line endings and a final newline.
    pub entry: String,
}

/// Append `fetched` to `file`. Nothing before the appended text changes. With `key_override`
/// only the key in the entry header is rewritten.
pub fn add(file: &BibFile, fetched: &Fetched, key_override: Option<&str>) -> Result<AddPlan> {
    let bibtex = fetched
        .bibtex
        .as_deref()
        .ok_or_else(|| Error::Provider("the provider returned no BibTeX".into()))?;
    let parsed = single_entry(bibtex)?;
    let text = parsed.bibtex.as_deref().unwrap_or(bibtex);

    let (key, text) = match key_override {
        None => (parsed.key, text.to_string()),
        Some(key) => (key.to_string(), replace_key(text, key)?),
    };

    let bib = file.parse();
    if bib.entries().any(|e| e.key.value == key) {
        return Err(Error::DuplicateKey(key));
    }

    let eol = file.line_ending.as_str();
    let mut entry = text.replace("\r\n", "\n").replace('\n', eol);
    entry.push_str(eol);

    let mut new_src = file.src.clone();
    if !new_src.is_empty() {
        if !new_src.ends_with('\n') {
            new_src.push_str(eol);
        }
        if !ends_with_blank_line(&new_src) {
            new_src.push_str(eol);
        }
    }
    new_src.push_str(&entry);
    Ok(AddPlan {
        new_src,
        key,
        entry,
    })
}

fn is_valid_key(key: &str) -> bool {
    !key.is_empty()
        && !key.chars().any(|c| {
            c.is_whitespace()
                || c.is_control()
                || matches!(c, ',' | '{' | '}' | '(' | ')' | '"' | '=' | '#' | '%')
        })
}

/// Replace only the key span of the single entry in `text`.
fn replace_key(text: &str, key: &str) -> Result<String> {
    if !is_valid_key(key) {
        return Err(Error::InvalidKey(key.to_string()));
    }
    let bib = bibx_parser::parse(text);
    let span = bib
        .entries()
        .next()
        .map(|e| e.key.span)
        .ok_or_else(|| Error::Parse {
            message: "no entry to rename".into(),
        })?;
    let (head, tail) = (text.get(..span.start), text.get(span.end..));
    match (head, tail) {
        (Some(head), Some(tail)) => Ok(format!("{head}{key}{tail}")),
        _ => Err(Error::Parse {
            message: "entry key is out of range".into(),
        }),
    }
}

/// Whether the last line of `src` (which ends with a newline) is blank.
fn ends_with_blank_line(src: &str) -> bool {
    let body = src.strip_suffix('\n').unwrap_or(src);
    let body = body.strip_suffix('\r').unwrap_or(body);
    body.rsplit('\n')
        .next()
        .is_some_and(|l| l.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTRY: &str = "@article{Maldacena:1997re,\n    title = \"{T}\",\n    year = \"1998\"\n}";

    fn fetched() -> Fetched {
        Fetched {
            key: "Maldacena:1997re".into(),
            bibtex: Some(format!("{ENTRY}\n")),
        }
    }

    fn run(src: &str, key: Option<&str>) -> Result<AddPlan> {
        add(
            &BibFile::from_source("x.bib", src.to_string()),
            &fetched(),
            key,
        )
    }

    #[test]
    fn appends_after_a_blank_line() {
        let plan = run("@book{a, title = {A}}\n", None).unwrap();
        assert_eq!(
            plan.new_src,
            format!("@book{{a, title = {{A}}}}\n\n{ENTRY}\n")
        );
        assert_eq!(plan.key, "Maldacena:1997re");
    }

    #[test]
    fn adds_the_missing_final_newline() {
        let plan = run("@book{a}", None).unwrap();
        assert_eq!(plan.new_src, format!("@book{{a}}\n\n{ENTRY}\n"));
    }

    #[test]
    fn reuses_an_existing_blank_line() {
        let plan = run("@book{a}\n\n", None).unwrap();
        assert_eq!(plan.new_src, format!("@book{{a}}\n\n{ENTRY}\n"));
    }

    #[test]
    fn empty_file_gets_just_the_entry() {
        let plan = run("", None).unwrap();
        assert_eq!(plan.new_src, format!("{ENTRY}\n"));
    }

    #[test]
    fn crlf_files_get_crlf_entries() {
        let plan = run("@book{a}\r\n", None).unwrap();
        let entry = ENTRY.replace('\n', "\r\n");
        assert_eq!(plan.new_src, format!("@book{{a}}\r\n\r\n{entry}\r\n"));
        assert_eq!(plan.entry, format!("{entry}\r\n"));
    }

    #[test]
    fn key_override_only_touches_the_key() {
        let plan = run("", Some("mypaper")).unwrap();
        assert_eq!(plan.key, "mypaper");
        assert_eq!(
            plan.new_src,
            ENTRY.replace("Maldacena:1997re", "mypaper") + "\n"
        );
    }

    #[test]
    fn duplicate_key_is_refused() {
        let err = run("@book{Maldacena:1997re}\n", None).unwrap_err();
        assert!(matches!(err, Error::DuplicateKey(k) if k == "Maldacena:1997re"));
        // --key avoids it
        assert!(run("@book{Maldacena:1997re}\n", Some("other")).is_ok());
        // ... unless the override collides too
        assert!(matches!(
            run("@book{other}\n", Some("other")),
            Err(Error::DuplicateKey(_))
        ));
    }

    #[test]
    fn invalid_override_is_refused() {
        assert!(matches!(run("", Some("a b")), Err(Error::InvalidKey(_))));
        assert!(matches!(run("", Some("")), Err(Error::InvalidKey(_))));
    }

    #[test]
    fn bibtex_is_required() {
        let f = Fetched {
            key: "k".into(),
            bibtex: None,
        };
        let file = BibFile::from_source("x.bib", String::new());
        assert!(matches!(add(&file, &f, None), Err(Error::Provider(_))));
    }
}

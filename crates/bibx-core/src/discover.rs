use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// A bib file to act on, and whether we guessed it (so output should name it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub path: PathBuf,
    pub auto_discovered: bool,
}

/// Choose the bib file: `flag`, else the single `*.bib` in `cwd` (not recursive).
pub fn resolve_file(flag: Option<&Path>, cwd: &Path) -> Result<Target> {
    if let Some(path) = flag {
        return Ok(Target {
            path: path.to_path_buf(),
            auto_discovered: false,
        });
    }
    let mut found = Vec::new();
    let entries = std::fs::read_dir(cwd).map_err(|e| Error::io(cwd, e))?;
    for entry in entries {
        let path = entry.map_err(|e| Error::io(cwd, e))?.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("bib"))
        {
            found.push(path);
        }
    }
    found.sort();
    match found.len() {
        0 => Err(Error::NoBibFile),
        1 => Ok(Target {
            path: relative_to(&found[0], cwd),
            auto_discovered: true,
        }),
        _ => Err(Error::ManyBibFiles(
            found.iter().map(|p| relative_to(p, cwd)).collect(),
        )),
    }
}

fn relative_to(path: &Path, cwd: &Path) -> PathBuf {
    path.strip_prefix(cwd).unwrap_or(path).to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(files: &[&str]) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        for f in files {
            std::fs::write(d.path().join(f), "").unwrap();
        }
        d
    }

    #[test]
    fn none() {
        let d = dir(&["notes.txt"]);
        assert!(matches!(
            resolve_file(None, d.path()),
            Err(Error::NoBibFile)
        ));
    }

    #[test]
    fn one_is_auto_discovered_and_named_relatively() {
        let d = dir(&["refs.bib", "notes.txt"]);
        let t = resolve_file(None, d.path()).unwrap();
        assert_eq!(
            t,
            Target {
                path: "refs.bib".into(),
                auto_discovered: true
            }
        );
    }

    #[test]
    fn many_lists_them_sorted() {
        let d = dir(&["b.bib", "a.bib"]);
        let Err(Error::ManyBibFiles(files)) = resolve_file(None, d.path()) else {
            panic!("expected ManyBibFiles");
        };
        assert_eq!(files, [PathBuf::from("a.bib"), PathBuf::from("b.bib")]);
    }

    #[test]
    fn flag_beats_discovery() {
        let d = dir(&["a.bib", "b.bib"]);
        let t = resolve_file(Some(Path::new("c.bib")), d.path()).unwrap();
        assert_eq!(
            t,
            Target {
                path: "c.bib".into(),
                auto_discovered: false
            }
        );
    }

    #[test]
    fn subdirectories_are_not_searched() {
        let d = dir(&[]);
        std::fs::create_dir(d.path().join("sub")).unwrap();
        std::fs::write(d.path().join("sub/x.bib"), "").unwrap();
        assert!(matches!(
            resolve_file(None, d.path()),
            Err(Error::NoBibFile)
        ));
    }
}

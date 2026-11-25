use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// A bib file to act on, and whether we guessed it (so output should name it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub path: PathBuf,
    pub auto_discovered: bool,
}

/// Choose the bib file(s): `-f` flags, then `BIBX_FILE`, then the single `*.bib` in `cwd`
/// (not recursive).
pub fn resolve_files(flags: &[PathBuf], env: Option<OsString>, cwd: &Path) -> Result<Vec<Target>> {
    if !flags.is_empty() {
        return Ok(explicit(flags.iter().cloned()));
    }
    if let Some(path) = env.filter(|v| !v.is_empty()) {
        return Ok(explicit([PathBuf::from(path)]));
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
        1 => Ok(vec![Target {
            path: relative_to(&found[0], cwd),
            auto_discovered: true,
        }]),
        _ => Err(Error::ManyBibFiles(
            found.iter().map(|p| relative_to(p, cwd)).collect(),
        )),
    }
}

fn explicit(paths: impl IntoIterator<Item = PathBuf>) -> Vec<Target> {
    paths
        .into_iter()
        .map(|path| Target {
            path,
            auto_discovered: false,
        })
        .collect()
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
            resolve_files(&[], None, d.path()),
            Err(Error::NoBibFile)
        ));
    }

    #[test]
    fn one_is_auto_discovered_and_named_relatively() {
        let d = dir(&["refs.bib", "notes.txt"]);
        let t = resolve_files(&[], None, d.path()).unwrap();
        assert_eq!(
            t,
            [Target {
                path: "refs.bib".into(),
                auto_discovered: true
            }]
        );
    }

    #[test]
    fn many_lists_them_sorted() {
        let d = dir(&["b.bib", "a.bib"]);
        let Err(Error::ManyBibFiles(files)) = resolve_files(&[], None, d.path()) else {
            panic!("expected ManyBibFiles");
        };
        assert_eq!(files, [PathBuf::from("a.bib"), PathBuf::from("b.bib")]);
    }

    #[test]
    fn env_beats_discovery_and_flag_beats_env() {
        let d = dir(&["a.bib", "b.bib"]);
        let t = resolve_files(&[], Some("c.bib".into()), d.path()).unwrap();
        assert_eq!(t[0].path, PathBuf::from("c.bib"));
        assert!(!t[0].auto_discovered);
        let t = resolve_files(
            &["x.bib".into(), "y.bib".into()],
            Some("c.bib".into()),
            d.path(),
        )
        .unwrap();
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].path, PathBuf::from("x.bib"));
    }

    #[test]
    fn empty_env_is_unset() {
        let d = dir(&["a.bib"]);
        let t = resolve_files(&[], Some("".into()), d.path()).unwrap();
        assert!(t[0].auto_discovered);
    }

    #[test]
    fn subdirectories_are_not_searched() {
        let d = dir(&[]);
        std::fs::create_dir(d.path().join("sub")).unwrap();
        std::fs::write(d.path().join("sub/x.bib"), "").unwrap();
        assert!(matches!(
            resolve_files(&[], None, d.path()),
            Err(Error::NoBibFile)
        ));
    }
}

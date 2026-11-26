use std::io::{self, Write};
use std::path::Path;

use crate::error::{Error, Result};

/// Write `bytes` to a temp file next to `path`, copy the original permissions and rename it
/// over `path`, so a crash never leaves a half-written file. Reject symbolic links.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let io = |e| Error::io(path, e);
    let mut tmp = tempfile::NamedTempFile::new_in(dir).map_err(io)?;
    tmp.write_all(bytes).map_err(io)?;
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.is_symlink() => {
            return Err(io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot write to a symbolic link, select its target instead",
            )));
        }
        Ok(meta) => std::fs::set_permissions(tmp.path(), meta.permissions()).map_err(io)?,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(io(err)),
    }
    tmp.persist(path).map_err(|e| Error::io(path, e.error))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn rejects_a_symlink_without_changing_it_or_its_target() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("refs.bib");
        let link = dir.path().join("alias.bib");
        std::fs::write(&target, "old").unwrap();
        std::os::unix::fs::symlink("refs.bib", &link).unwrap();

        let error = atomic_write(&link, b"new").unwrap_err();
        assert!(error.to_string().contains("symbolic link"));
        assert_eq!(std::fs::read_link(&link).unwrap(), Path::new("refs.bib"));
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "old");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
    }

    #[test]
    fn replaces_content_and_keeps_permissions() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a.bib");
        std::fs::write(&p, "old").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o640)).unwrap();
        }
        atomic_write(&p, b"new").unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "new");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&p).unwrap().permissions().mode() & 0o777,
                0o640
            );
        }
        assert_eq!(std::fs::read_dir(d.path()).unwrap().count(), 1);
    }
}

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

/// Like [`atomic_write`], but refuse with [`Error::ConcurrentEdit`] unless the file on disk
/// still holds exactly `expected`, the text the new bytes were planned from.
pub fn atomic_write_if_unchanged(path: &Path, expected: &str, bytes: &[u8]) -> Result<()> {
    let current = std::fs::read(path).map_err(|e| Error::io(path, e))?;
    if current != expected.as_bytes() {
        return Err(Error::ConcurrentEdit(path.to_path_buf()));
    }
    atomic_write(path, bytes)
}

/// Create `path` with `bytes`, refusing with [`Error::ConcurrentEdit`] if it exists by now.
pub fn create_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            return Err(Error::ConcurrentEdit(path.to_path_buf()));
        }
        Err(err) => return Err(Error::io(path, err)),
    };
    file.write_all(bytes).map_err(|e| Error::io(path, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guarded_write_refuses_a_file_that_changed_since_it_was_read() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a.bib");
        std::fs::write(&p, "old\nconcurrent\n").unwrap();

        let err = atomic_write_if_unchanged(&p, "old\n", b"old\nnew\n").unwrap_err();
        assert!(matches!(err, Error::ConcurrentEdit(path) if path == p));
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "old\nconcurrent\n");
        assert_eq!(std::fs::read_dir(d.path()).unwrap().count(), 1);
    }

    #[test]
    fn create_new_writes_a_new_file_and_refuses_an_existing_one() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a.bib");
        create_new(&p, b"new\n").unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "new\n");
        let err = create_new(&p, b"other\n").unwrap_err();
        assert!(matches!(err, Error::ConcurrentEdit(path) if path == p));
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "new\n");
    }

    #[test]
    fn guarded_write_replaces_an_unchanged_file() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a.bib");
        std::fs::write(&p, "old\n").unwrap();
        atomic_write_if_unchanged(&p, "old\n", b"old\nnew\n").unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "old\nnew\n");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_symlink_without_changing_it_or_its_target() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("refs.bib");
        let link = dir.path().join("alias.bib");
        std::fs::write(&target, "old").unwrap();
        std::os::unix::fs::symlink("refs.bib", &link).unwrap();

        let error = atomic_write(&link, b"new").unwrap_err();
        let cause = std::error::Error::source(&error).unwrap();
        assert!(cause.to_string().contains("symbolic link"));
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

use std::io::Write;
use std::path::Path;

use crate::error::{Error, Result};

/// Write `bytes` to a temp file next to `path`, copy the original permissions and rename it
/// over `path`, so a crash never leaves a half-written file.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let io = |e| Error::io(path, e);
    let mut tmp = tempfile::NamedTempFile::new_in(dir).map_err(io)?;
    tmp.write_all(bytes).map_err(io)?;
    if let Ok(meta) = std::fs::metadata(path) {
        std::fs::set_permissions(tmp.path(), meta.permissions()).map_err(io)?;
    }
    tmp.persist(path).map_err(|e| Error::io(path, e.error))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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

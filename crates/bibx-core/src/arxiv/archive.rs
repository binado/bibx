use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

use flate2::bufread::MultiGzDecoder;

use super::{DOWNLOAD_LIMIT, PullError, Result, download_limit};

const TAR_BLOCK: usize = 512;
const EXPANDED_LIMIT: u64 = 512 * 1024 * 1024;
const ENTRY_LIMIT: usize = 10_000;
const BIB_LIMIT: u64 = 32 * 1024 * 1024;

#[derive(Default)]
pub(super) struct Selection {
    paths: Vec<Vec<u8>>,
    has_bbl: bool,
    bytes: Option<Result<Vec<u8>>>,
    matches: usize,
}

impl Selection {
    pub(super) fn finish(mut self, member: Option<&[u8]>) -> Result<Vec<u8>> {
        if self.paths.is_empty() {
            return Err(if self.has_bbl {
                PullError::BblOnly
            } else {
                PullError::NoBibliography
            });
        }
        match member {
            Some(path) if self.matches == 0 => Err(PullError::NoMember(display_path(path))),
            Some(path) if self.matches > 1 => Err(PullError::DuplicateMember(display_path(path))),
            None if self.paths.len() > 1 => {
                self.paths.sort_unstable();
                Err(PullError::Ambiguous(self.paths))
            }
            _ => self.bytes.ok_or(PullError::NoBibliography)?,
        }
    }
}

pub(super) fn member_path(mut path: &[u8]) -> Option<&[u8]> {
    while let Some(rest) = path.strip_prefix(b"./") {
        path = rest;
    }
    if path.contains(&0)
        || path.contains(&b'\\')
        || path
            .split(|byte| *byte == b'/')
            .any(|part| matches!(part, b"" | b"." | b".."))
    {
        return None;
    }
    Some(path)
}

fn display_path(path: &[u8]) -> String {
    match std::str::from_utf8(path) {
        Ok(path) if !path.chars().any(char::is_control) => path.to_owned(),
        Ok(path) => format!("{path:?}"),
        Err(_) => path.escape_ascii().to_string(),
    }
}

pub(super) fn list_paths(paths: &[Vec<u8>]) -> String {
    paths
        .iter()
        .map(|p| format!("\n  {}", display_path(p)))
        .collect()
}

pub(super) fn scan(file: &File, member: Option<&[u8]>) -> Result<Selection> {
    if file.metadata().map_err(PullError::InvalidArchive)?.len() > DOWNLOAD_LIMIT {
        return Err(download_limit());
    }
    scan_source(
        BufReader::new(file),
        member,
        EXPANDED_LIMIT,
        ENTRY_LIMIT,
        BIB_LIMIT,
    )
}

fn scan_source(
    mut compressed: impl BufRead,
    member: Option<&[u8]>,
    expanded_limit: u64,
    entry_limit: usize,
    bib_limit: u64,
) -> Result<Selection> {
    if !compressed
        .fill_buf()
        .map_err(PullError::InvalidArchive)?
        .starts_with(&[0x1f, 0x8b])
    {
        return Err(PullError::UnsupportedSource);
    }
    let mut decoded = MultiGzDecoder::new(compressed).take(expanded_limit + 1);
    let result = scan_decoded(&mut decoded, member, entry_limit, bib_limit);
    if decoded.limit() == 0 {
        return Err(PullError::Limit("decompressed source", "512 MiB"));
    }
    result
}

fn scan_decoded(
    mut decoded: impl Read,
    member: Option<&[u8]>,
    entry_limit: usize,
    bib_limit: u64,
) -> Result<Selection> {
    let mut prefix = Vec::with_capacity(TAR_BLOCK);
    decoded
        .by_ref()
        .take(TAR_BLOCK as u64)
        .read_to_end(&mut prefix)
        .map_err(PullError::InvalidArchive)?;
    if looks_like_tar(&prefix) {
        return scan_tar(
            prefix.as_slice().chain(decoded),
            member,
            entry_limit,
            bib_limit,
        );
    }
    if prefix.starts_with(b"%PDF-") || prefix.starts_with(b"%!PS") {
        return Err(PullError::UnsupportedSource);
    }
    if looks_like_tex(prefix.as_slice().chain(decoded)).map_err(PullError::InvalidArchive)? {
        Ok(Selection::default())
    } else {
        Err(PullError::UnsupportedSource)
    }
}

fn looks_like_tex(mut reader: impl Read) -> io::Result<bool> {
    let mut block = [0; 8192];
    let mut has_control_word = false;
    let mut after_backslash = false;
    loop {
        let count = reader.read(&mut block)?;
        if count == 0 {
            return Ok(has_control_word);
        }
        for &byte in &block[..count] {
            if byte == 0 {
                return Ok(false);
            }
            has_control_word |= after_backslash && byte.is_ascii_alphabetic();
            after_backslash = byte == b'\\';
        }
    }
}

fn looks_like_tar(prefix: &[u8]) -> bool {
    if prefix.get(257..262) == Some(b"ustar") {
        return true;
    }
    if prefix.len() != TAR_BLOCK {
        return false;
    }
    if prefix.iter().all(|&byte| byte == 0) {
        return true;
    }
    let header = tar::Header::from_byte_slice(prefix);
    let mut checksummed = header.clone();
    checksummed.set_cksum();
    header
        .cksum()
        .is_ok_and(|sum| checksummed.cksum().ok() == Some(sum))
}

fn scan_tar(
    mut reader: impl Read,
    member: Option<&[u8]>,
    entry_limit: usize,
    bib_limit: u64,
) -> Result<Selection> {
    let mut archive = tar::Archive::new(&mut reader);
    let mut selection = Selection::default();
    for (index, entry) in archive
        .entries()
        .map_err(PullError::InvalidArchive)?
        .enumerate()
    {
        if index == entry_limit {
            return Err(PullError::Limit("archive entry count", "10,000"));
        }
        let mut entry = entry.map_err(PullError::InvalidArchive)?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path_bytes();
        let Some(path) = member_path(&path) else {
            continue;
        };
        let suffix = &path[path.len().saturating_sub(4)..];
        if suffix.eq_ignore_ascii_case(b".bbl") {
            selection.has_bbl = true;
        }
        if !suffix.eq_ignore_ascii_case(b".bib") {
            continue;
        }
        let wanted = match member {
            Some(member) => path == member,
            None => selection.paths.is_empty(),
        };
        selection.paths.push(path.to_vec());
        if wanted {
            selection.matches += 1;
            if selection.matches == 1 {
                selection.bytes = Some(if entry.size() > bib_limit {
                    Err(PullError::Limit("selected bibliography", "32 MiB"))
                } else {
                    let mut bytes = Vec::new();
                    entry
                        .read_to_end(&mut bytes)
                        .map_err(PullError::InvalidArchive)?;
                    Ok(bytes)
                });
            }
        }
    }
    check_tar_padding(reader).map_err(PullError::InvalidArchive)?;
    Ok(selection)
}

fn check_tar_padding(mut reader: impl Read) -> io::Result<()> {
    let mut block = [0; 8192];
    reader.read_exact(&mut block[..TAR_BLOCK])?;
    let mut count = TAR_BLOCK;
    while count != 0 {
        if block[..count].iter().any(|&byte| byte != 0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "nonzero data after the tar end marker",
            ));
        }
        count = reader.read(&mut block)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::{Compression, write::GzEncoder};

    use super::*;

    fn gzip(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }

    fn tar_with_bib_and_link() -> Vec<u8> {
        let mut tar = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_old();
        header.set_size(4);
        tar.append_data(&mut header, "refs.bib", &b"refs"[..])
            .unwrap();
        let mut link = tar::Header::new_gnu();
        link.set_entry_type(tar::EntryType::Symlink);
        link.set_size(0);
        tar.append_link(&mut link, "link.bib", "refs.bib").unwrap();
        tar.into_inner().unwrap()
    }

    #[test]
    fn sees_regular_members_in_legacy_tar_and_ignores_links() {
        let compressed = gzip(&tar_with_bib_and_link());
        let selection = scan_source(
            compressed.as_slice(),
            None,
            EXPANDED_LIMIT,
            ENTRY_LIMIT,
            BIB_LIMIT,
        )
        .unwrap();
        assert_eq!(selection.finish(None).unwrap(), b"refs");
    }

    #[test]
    fn rejects_a_tar_truncated_after_a_complete_bibliography() {
        let mut tar = tar_with_bib_and_link();
        tar.truncate(tar.len() - TAR_BLOCK);
        let compressed = gzip(&tar);
        assert!(matches!(
            scan_source(
                compressed.as_slice(),
                None,
                EXPANDED_LIMIT,
                ENTRY_LIMIT,
                BIB_LIMIT
            ),
            Err(PullError::InvalidArchive(_))
        ));
    }

    #[test]
    fn enforces_archive_and_selected_bibliography_limits() {
        let tar = tar_with_bib_and_link();
        let compressed = gzip(&tar);
        let source = compressed.as_slice();
        assert!(matches!(
            scan_source(source, None, tar.len() as u64 - 1, ENTRY_LIMIT, BIB_LIMIT),
            Err(PullError::Limit("decompressed source", _))
        ));
        assert!(matches!(
            scan_source(source, None, EXPANDED_LIMIT, 1, BIB_LIMIT),
            Err(PullError::Limit("archive entry count", _))
        ));
        let selection = scan_source(source, None, tar.len() as u64, 2, 3).unwrap();
        assert!(matches!(
            selection.finish(None),
            Err(PullError::Limit("selected bibliography", _))
        ));
        let selection = scan_source(source, None, tar.len() as u64, 2, 4).unwrap();
        assert_eq!(selection.finish(None).unwrap(), b"refs");
    }
}

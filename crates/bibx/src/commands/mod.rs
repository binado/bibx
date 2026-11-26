pub mod add;
pub mod check;
pub mod list;
pub mod remove;
pub mod show;

use std::io::{self, ErrorKind, Write};
use std::path::PathBuf;

use anyhow::Context;
use bibx_core::{BibFile, Target, resolve_file};
use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct FileArgs {
    /// BibTeX file to use. Falls back to the only *.bib file in the current directory.
    #[arg(short, long, value_name = "FILE", env = "BIBX_FILE")]
    file: Option<PathBuf>,
}

impl FileArgs {
    fn target(&self) -> anyhow::Result<Target> {
        let cwd = std::env::current_dir()?;
        Ok(resolve_file(self.file.as_deref(), &cwd)?)
    }
}

pub fn load(target: &Target) -> anyhow::Result<BibFile> {
    Ok(BibFile::load(&target.path)?)
}

fn write_stdout(command: &str, out: &str) -> anyhow::Result<()> {
    let mut stdout =
        stdout_writer().with_context(|| format!("opening stdout for {command} output"))?;
    match stdout
        .write_all(out.as_bytes())
        .and_then(|()| stdout.flush())
    {
        Err(err) if err.kind() == ErrorKind::BrokenPipe => Ok(()),
        result => result.with_context(|| format!("writing {command} output to stdout")),
    }
}

#[cfg(unix)]
fn stdout_writer() -> io::Result<impl Write> {
    use std::os::fd::AsFd;

    // Stdout treats EBADF as success. Writing through a duplicate reports that error.
    let fd = io::stdout().as_fd().try_clone_to_owned()?;
    Ok(std::fs::File::from(fd))
}

#[cfg(not(unix))]
fn stdout_writer() -> io::Result<impl Write> {
    Ok(io::stdout().lock())
}

/// ` <prefix> <file>` when the file was auto-discovered, so a wrong guess is visible.
pub fn named(target: &Target, prefix: &str) -> String {
    if target.auto_discovered {
        format!("{prefix} {}", target.path.display())
    } else {
        String::new()
    }
}

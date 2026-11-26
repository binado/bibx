pub mod add;
pub mod list;
pub mod remove;

use std::path::PathBuf;

use bibx_core::{BibFile, Target, resolve_file};
use clap::Args;

#[derive(Debug, Args)]
struct FileArgs {
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

/// ` <prefix> <file>` when the file was auto-discovered, so a wrong guess is visible.
pub fn named(target: &Target, prefix: &str) -> String {
    if target.auto_discovered {
        format!("{prefix} {}", target.path.display())
    } else {
        String::new()
    }
}

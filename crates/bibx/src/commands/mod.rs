pub mod add;
pub mod list;

use std::path::PathBuf;

use bibx_core::{BibFile, Target, resolve_files};
use clap::CommandFactory;
use clap::error::ErrorKind;

use crate::cli::Cli;

/// The bib files to act on, per the flag/env/discovery rules.
pub fn targets(files: &[PathBuf]) -> anyhow::Result<Vec<Target>> {
    let cwd = std::env::current_dir()?;
    Ok(resolve_files(files, std::env::var_os("BIBX_FILE"), &cwd)?)
}

/// For commands that act on exactly one file; more than one `-f` is a usage error.
pub fn single_target(files: &[PathBuf]) -> anyhow::Result<Target> {
    if files.len() > 1 {
        Cli::command()
            .error(
                ErrorKind::ArgumentConflict,
                "this command takes a single -f/--file",
            )
            .exit();
    }
    Ok(targets(files)?.remove(0))
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

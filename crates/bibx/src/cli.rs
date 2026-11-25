use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::commands::list::ListArgs;

/// Manage a BibTeX file: list its entries (and, soon, add and remove them).
///
/// The bib file is chosen from `-f/--file`, then the `BIBX_FILE` environment variable, then
/// the only `*.bib` file in the current directory. The file is edited in place with small
/// diffs; it is never re-serialized.
///
/// Data goes to stdout and messages to stderr. Exit codes: 0 ok, 1 error, 2 usage,
/// 3 duplicate key, 4 ambiguous match.
#[derive(Debug, Parser)]
#[command(name = "bibx", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,

    /// The bib file to use (see above for the default).
    #[arg(short, long, global = true, value_name = "FILE")]
    pub file: Vec<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List the entries of the bib file.
    List(ListArgs),
}

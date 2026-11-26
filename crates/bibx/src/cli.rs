use clap::{Parser, Subcommand};

use crate::commands::add::AddArgs;
use crate::commands::list::ListArgs;
use crate::commands::remove::RemoveArgs;

/// Manage a BibTeX file: add, remove and list its entries.
///
/// The bib file is chosen from the subcommand's `-f/--file` option, then the `BIBX_FILE`
/// environment variable, then the only `*.bib` file in the current directory. The file is
/// edited in place with small diffs; it is never re-serialized.
///
/// Data goes to stdout and messages to stderr. Exit codes: 0 ok, 1 error, 2 usage,
/// 3 duplicate key, 4 ambiguous match.
#[derive(Debug, Parser)]
#[command(name = "bibx", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Fetch an entry from a provider and append it to the bib file.
    ///
    /// The identifier is `<kind>:<value>` with kind `eprint` (also `arxiv`), `doi` or
    /// `inspire` (a record id or a texkey). Use `url:` for arXiv abstract or PDF links
    /// and INSPIRE literature links. Examples are `url:https://arxiv.org/abs/2005.12345`
    /// and `url:https://inspirehep.net/literature/451647`.
    ///
    /// The entry is appended as received, with an optional citation key override via
    /// `--key`. If the key already exists in the file, the command exits with code 3.
    Add(AddArgs),
    /// Delete one entry from the bib file.
    Remove(RemoveArgs),
    /// List the entries of the bib file.
    List(ListArgs),
}

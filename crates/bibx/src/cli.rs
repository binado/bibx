use clap::{Parser, Subcommand};

use crate::commands::FileArgs;
use crate::commands::add::AddArgs;
use crate::commands::list::ListArgs;
use crate::commands::pull::PullArgs;
use crate::commands::remove::RemoveArgs;
use crate::commands::show::ShowArgs;

/// Manage and validate BibTeX files and pull bibliographies from arXiv.
///
/// Commands that use a local bibliography choose the file from `-f/--file`, then the
/// `BIBX_FILE` environment variable, then the only `*.bib` file in the current directory.
/// Edits are applied in place as small diffs.
///
/// Data goes to stdout and messages to stderr. Exit codes: 0 ok, 1 error, 2 usage,
/// 3 duplicate (key, eprint or DOI), 4 ambiguous match.
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
    /// `inspire` (a record id or a texkey). A bare arXiv id is an eprint, and arXiv abstract
    /// or PDF links and INSPIRE literature links are recognized, with or without `url:`.
    /// Examples are `2005.12345`, `https://arxiv.org/abs/2005.12345` and
    /// `https://inspirehep.net/literature/451647`.
    ///
    /// The entry is appended as received, with an optional citation key override via
    /// `--key`. A missing file named by `-f` or `BIBX_FILE` is created. If the key already
    /// exists in the file (ignoring case), or an entry there has the same eprint or DOI,
    /// nothing is written and the command exits with code 3; `--allow-duplicate` adds a
    /// repeated paper anyway.
    Add(AddArgs),
    /// Delete one entry from the bib file.
    Remove(RemoveArgs),
    /// List the entries of the bib file.
    List(ListArgs),
    /// Print one entry as stored in the bib file.
    Show(ShowArgs),
    /// Check the bib file for syntax errors and duplicate citation keys, eprints and DOIs.
    ///
    /// Keys that differ only in case count as duplicates. Reports all findings on stderr
    /// without editing the file. Exits with code 0 for a clean file, 3 for duplicates alone,
    /// 1 for syntax or file errors, and 2 for invalid usage.
    Check(FileArgs),
    /// Write a submitted arXiv bibliography to stdout.
    Pull(PullArgs),
}

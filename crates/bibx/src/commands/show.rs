use bibx_core::Identifier;
use bibx_core::ops::show::show;
use clap::Args;

use super::{FileArgs, load, write_stdout};

/// Print one entry as stored in the bib file.
///
/// A bare value is a citation key. Use `eprint:` or `arxiv:` to match the `eprint` field,
/// `doi:` to match the `doi` field, or `key:` to match the key, including keys that start
/// with a known prefix.
///
/// Stdout contains the entry from its @ through its closing delimiter, without an added
/// newline. If no entry matches, the command exits with code 1. If several entries
/// match, the matching keys are listed on stderr and the command exits with code 4.
#[derive(Debug, Args)]
pub struct ShowArgs {
    /// Which entry, e.g. `Author:2020abc`, `eprint:2005.12345` or `doi:10.1103/...`.
    id: String,

    #[command(flatten)]
    file: FileArgs,
}

pub fn run(args: ShowArgs) -> anyhow::Result<()> {
    let id = Identifier::parse(&args.id);
    let file = load(&args.file.target()?)?;
    write_stdout("show", show(&file, &id)?)
}

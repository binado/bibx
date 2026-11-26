use bibx_core::Identifier;
use bibx_core::ops::remove::remove;
use bibx_core::write::atomic_write;
use clap::Args;

use super::{load, named, single_target};

/// Delete one entry from the bib file.
///
/// A bare value is a citation key. Use `eprint:` or `arxiv:` to match the `eprint` field,
/// `doi:` to match the `doi` field, or `key:` to match the key, including keys that start
/// with a known prefix. Only the entry's own lines are deleted. The removed
/// entry is always printed to stdout. If several entries match, nothing is removed and the
/// matches are listed (exit code 4).
#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// Which entry, e.g. `Author:2020abc`, `eprint:2005.12345` or `doi:10.1103/...`.
    id: String,

    /// Print the entry that would be removed and write nothing.
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: RemoveArgs, files: &[std::path::PathBuf]) -> anyhow::Result<()> {
    let id = Identifier::parse(&args.id);
    let target = single_target(files)?;
    let file = load(&target)?;
    let plan = remove(&file, &id)?;
    println!("{}", plan.removed_text);
    if args.dry_run {
        eprintln!("would remove {}{}", plan.key, named(&target, " from"));
        return Ok(());
    }
    atomic_write(&file.path, plan.new_src.as_bytes())?;
    eprintln!("removed {}{}", plan.key, named(&target, " from"));
    Ok(())
}

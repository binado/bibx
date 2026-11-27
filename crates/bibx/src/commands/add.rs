use bibx_core::ops::add::add;
use bibx_core::provider::{Source, provider_for};
use bibx_core::write::atomic_write_if_unchanged;
use bibx_core::{Error, Identifier, Kind};
use clap::{Args, ValueEnum};

use super::{FileArgs, load};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SourceArg {
    Inspire,
}

impl From<SourceArg> for Source {
    fn from(s: SourceArg) -> Source {
        match s {
            SourceArg::Inspire => Source::Inspire,
        }
    }
}

/// Fetch an entry from a provider, append it to the bib file, and print it to stdout.
#[derive(Debug, Args)]
pub struct AddArgs {
    /// What to fetch, e.g. `eprint:2005.12345`, `2005.12345` or an arXiv or INSPIRE URL.
    id: String,

    /// Where to fetch it from.
    #[arg(long, value_enum, default_value = "inspire")]
    source: SourceArg,

    /// Use this citation key instead of the provider's.
    #[arg(long)]
    key: Option<String>,

    /// Print the entry that would be appended and write nothing.
    #[arg(long)]
    dry_run: bool,

    /// Add the entry even if the file already has one with the same eprint or DOI.
    #[arg(long)]
    allow_duplicate: bool,

    #[command(flatten)]
    file: FileArgs,
}

pub fn run(args: AddArgs) -> anyhow::Result<()> {
    let id = Identifier::from_fetch_arg(&args.id)?;
    if id.kind() == Kind::Key {
        return Err(Error::UnfetchableKey(id.value().to_string()).into());
    }
    let target = args.file.target()?;
    let fetched = provider_for(args.source.into()).fetch(&id)?;
    let file = load(&target)?;
    let plan = add(&file, &fetched, args.key.as_deref(), args.allow_duplicate)?;
    if args.dry_run {
        eprintln!("[dry run] Skipping write to {}", file.path.display());
    } else {
        atomic_write_if_unchanged(&file.path, &file.src, plan.new_src.as_bytes())?;
    }
    print!("{}", plan.entry);
    Ok(())
}

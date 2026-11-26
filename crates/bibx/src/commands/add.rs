use bibx_core::ops::add::add;
use bibx_core::provider::{Source, provider_for};
use bibx_core::write::atomic_write;
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
    /// What to fetch, e.g. `eprint:2005.12345`.
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

    #[command(flatten)]
    file: FileArgs,
}

pub fn run(args: AddArgs) -> anyhow::Result<()> {
    let id = match args.id.strip_prefix("url:") {
        Some(url) => Identifier::from_url(url)?,
        None => Identifier::parse(&args.id),
    };
    if id.kind == Kind::Key {
        return Err(Error::UnfetchableKey(id.value).into());
    }
    let target = args.file.target()?;
    let file = load(&target)?;
    let fetched = provider_for(args.source.into()).fetch(&id)?;
    let plan = add(&file, &fetched, args.key.as_deref())?;
    if args.dry_run {
        eprintln!("[dry run] Skipping write to {}", file.path.display());
    } else {
        atomic_write(&file.path, plan.new_src.as_bytes())?;
    }
    print!("{}", plan.entry);
    Ok(())
}

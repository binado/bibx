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

/// Fetch an entry from a provider and append it to the bib file.
///
/// The identifier is `<kind>:<value>` with kind `eprint`, `doi` or `inspire` (a record id
/// or a texkey), e.g. `eprint:hep-th/9711200` or `doi:10.1103/PhysRevD.102.123456`. The
/// entry is appended exactly as received; with --key only its citation key changes. If the
/// key already exists in the file, nothing is added (exit code 3).
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
    let id = Identifier::parse(&args.id);
    if id.kind == Kind::Key {
        return Err(Error::UnfetchableKey(id.value).into());
    }
    let target = args.file.target()?;
    let file = load(&target)?;
    let fetched = provider_for(args.source.into()).fetch(&id)?;
    let plan = add(&file, &fetched, args.key.as_deref())?;
    if args.dry_run {
        print!("{}", plan.entry);
        return Ok(());
    }
    atomic_write(&file.path, plan.new_src.as_bytes())?;
    eprintln!("added {}{}", plan.key, super::named(&target, " to"));
    Ok(())
}

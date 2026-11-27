use std::path::PathBuf;

use anyhow::bail;
use bibx_core::arxiv::{Arxiv, ArxivId};
use bibx_core::write::atomic_write;
use clap::Args;

use super::write_stdout;

#[derive(Debug, Args)]
#[command(
    after_help = "A selected .bib file is written byte for byte. If there are several .bib files,\n\
                  bibx lists their paths on stderr and exits with code 4. A .bbl-only submission\n\
                  exits with an error.\n\n\
                  With --output, the file is written only after a successful pull, and an existing\n\
                  file is refused (exit code 1) unless --force is given. Nothing goes to stdout.\n\n\
                  Explicit versions use the platform user cache. Unversioned IDs fetch the latest source."
)]
pub struct PullArgs {
    /// arXiv ID, arxiv:<id>, eprint:<id>, or url:<arXiv abstract or PDF URL>.
    #[arg(value_name = "ARXIV_ID")]
    id: ArxivId,

    /// Select a .bib by its full archive-relative path, using / separators.
    #[arg(long, value_name = "MEMBER")]
    path: Option<PathBuf>,

    /// Write the bibliography to PATH instead of stdout, only after a successful pull.
    #[arg(short, long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// Overwrite --output if it already exists.
    #[arg(long, requires = "output")]
    force: bool,
}

pub fn run(args: PullArgs) -> anyhow::Result<()> {
    if let Some(output) = args.output.as_deref().filter(|_| !args.force)
        && output.symlink_metadata().is_ok()
    {
        bail!(
            "{} already exists, pass --force to overwrite it",
            output.display()
        );
    }
    let bytes = Arxiv::new().pull(&args.id, args.path.as_deref())?;
    match args.output {
        Some(output) => {
            atomic_write(&output, &bytes)?;
            eprintln!("wrote {}", output.display());
            Ok(())
        }
        None => write_stdout("pull", bytes),
    }
}

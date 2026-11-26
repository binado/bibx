use std::path::PathBuf;

use bibx_core::arxiv::{Arxiv, ArxivId};
use clap::Args;

use super::write_stdout;

#[derive(Debug, Args)]
#[command(
    after_help = "A selected .bib file is written byte for byte. If there are several .bib files,\n\
                  bibx lists their paths on stderr and exits with code 4. A .bbl-only submission\n\
                  exits with an error.\n\n\
                  Explicit versions use the platform user cache. Unversioned IDs fetch the latest source."
)]
pub struct PullArgs {
    /// arXiv ID, arxiv:<id>, eprint:<id>, or url:<arXiv abstract or PDF URL>.
    #[arg(value_name = "ARXIV_ID")]
    id: ArxivId,

    /// Select a .bib by its full archive-relative path, using / separators.
    #[arg(long, value_name = "MEMBER")]
    path: Option<PathBuf>,
}

pub fn run(args: PullArgs) -> anyhow::Result<()> {
    let bytes = Arxiv::new().pull(&args.id, args.path.as_deref())?;
    write_stdout("pull", bytes)
}

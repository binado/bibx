use std::process::ExitCode;

use bibx_core::ops::check::{DiagnosticKind, check};

use super::{FileArgs, load_quiet};

pub(crate) fn run(args: FileArgs) -> anyhow::Result<ExitCode> {
    let file = load_quiet(&args.target()?)?;
    let path = file.path.display();
    let mut code = 0;
    for diagnostic in check(&file) {
        let location = diagnostic.location;
        let message = match diagnostic.kind {
            DiagnosticKind::Syntax(message) => {
                code = 1;
                message
            }
            DiagnosticKind::DuplicateKey { key, first } => {
                if code == 0 {
                    code = 3;
                }
                format!(
                    "duplicate citation key `{key}` (first occurrence at {path}:{}:{})",
                    first.line, first.column
                )
            }
        };
        eprintln!(
            "{path}:{}:{}: error: {message}",
            location.line, location.column
        );
    }
    Ok(ExitCode::from(code))
}

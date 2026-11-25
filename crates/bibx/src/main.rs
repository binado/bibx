mod cli;
mod commands;

use std::process::ExitCode;

use bibx_core::Error;
use clap::Parser;

use cli::{Cli, Command};

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.cmd {
        Command::List(args) => commands::list::run(args, &cli.file),
    }
}

/// 1 for any error, 3 for a duplicate key and 4 for an ambiguous match; usage errors are
/// exit code 2, from clap.
fn exit_code(err: &anyhow::Error) -> u8 {
    match err.downcast_ref::<Error>() {
        Some(Error::DuplicateKey(_)) => 3,
        Some(Error::Ambiguous { .. }) => 4,
        _ => 1,
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(exit_code(&err))
        }
    }
}

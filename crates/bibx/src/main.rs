mod cli;
mod commands;

use std::process::ExitCode;

use bibx_core::Error;
use clap::Parser;

use cli::{Cli, Command};

fn run(cli: Cli) -> anyhow::Result<ExitCode> {
    match cli.cmd {
        Command::Add(args) => commands::add::run(args),
        Command::Remove(args) => commands::remove::run(args),
        Command::List(args) => commands::list::run(args),
        Command::Show(args) => commands::show::run(args),
        Command::Check(args) => return commands::check::run(args),
    }?;
    Ok(ExitCode::SUCCESS)
}

/// 1 for any error, 3 for a duplicate key and 4 for an ambiguous match; usage errors are
/// exit code 2, from clap.
fn exit_code(err: &anyhow::Error) -> u8 {
    match err.downcast_ref::<Error>() {
        Some(Error::DuplicateKey(_)) => 3,
        Some(Error::Ambiguous { .. } | Error::AmbiguousInspire(_)) => 4,
        _ => 1,
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::from(exit_code(&err))
        }
    }
}

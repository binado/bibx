mod cli;
mod commands;

use std::process::ExitCode;

use bibx_core::Error;
use bibx_core::arxiv::PullError;
use clap::Parser;

use cli::{Cli, Command};

fn run(cli: Cli) -> anyhow::Result<ExitCode> {
    match cli.cmd {
        Command::Add(args) => commands::add::run(args),
        Command::Remove(args) => commands::remove::run(args),
        Command::List(args) => commands::list::run(args),
        Command::Show(args) => commands::show::run(args),
        Command::Check(args) => return commands::check::run(args),
        Command::Pull(args) => commands::pull::run(args),
    }?;
    Ok(ExitCode::SUCCESS)
}

fn exit_code(err: &anyhow::Error) -> u8 {
    if let Some(error) = err.downcast_ref::<PullError>() {
        return match error {
            PullError::InvalidMember => 2,
            PullError::Ambiguous(_) => 4,
            _ => 1,
        };
    }
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

#![cfg(unix)]

use std::fs::File;
use std::io;
use std::process::{Command, Stdio};

const BIB: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/record.bib");

fn list() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bibx"));
    command.args(["list", "-f", BIB, "--format", "json"]);
    command
}

#[test]
fn list_reports_unwritable_stdout() {
    let stdout = Stdio::from(File::open(BIB).unwrap());
    let output = list().stdout(stdout).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("writing list output to stdout"));
}

#[test]
fn list_succeeds_if_the_pipe_reader_closes() {
    let (reader, writer) = io::pipe().unwrap();
    drop(reader);
    let output = list().stdout(writer).output().unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}

//! End-to-end tests that need the INSPIRE mock server, which mdtest cannot start.

use std::fs;

use assert_cmd::Command;
use bibx_core::provider::testing::{MockServer, Reply};

const RECORD: &str = include_str!("fixtures/record.bib");
const ROUTE: &str = "/api/arxiv/hep-th/9711200?format=bibtex";

fn server() -> MockServer {
    MockServer::start(vec![(ROUTE.into(), Reply::bibtex(RECORD))])
}

fn bibx(server: &MockServer, dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("bibx").unwrap();
    cmd.current_dir(dir)
        .env_clear()
        .env("BIBX_INSPIRE_URL", &server.url);
    cmd
}

#[test]
fn add_appends_the_entry_and_names_an_auto_discovered_file() {
    let (server, dir) = (server(), tempfile::tempdir().unwrap());
    let path = dir.path().join("refs.bib");
    fs::write(&path, "@book{a, title = {A}}\n").unwrap();
    bibx(&server, dir.path())
        .args(["add", "eprint:hep-th/9711200"])
        .assert()
        .success()
        .stdout("")
        .stderr("added Maldacena:1997re to refs.bib\n");
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        format!(
            "@book{{a, title = {{A}}}}\n\n{}",
            RECORD.trim_end().to_string() + "\n"
        )
    );
}

#[test]
fn add_with_an_explicit_file_does_not_name_it() {
    let (server, dir) = (server(), tempfile::tempdir().unwrap());
    fs::write(dir.path().join("refs.bib"), "").unwrap();
    bibx(&server, dir.path())
        .args([
            "add",
            "-f",
            "refs.bib",
            "arxiv:hep-th/9711200v2",
            "--key",
            "mine",
        ])
        .assert()
        .success()
        .stderr("added mine\n");
    let out = fs::read_to_string(dir.path().join("refs.bib")).unwrap();
    assert!(out.starts_with("@article{mine,\n"));
}

#[test]
fn dry_run_prints_the_entry_and_writes_nothing() {
    let (server, dir) = (server(), tempfile::tempdir().unwrap());
    let path = dir.path().join("refs.bib");
    fs::write(&path, "@book{a}\n").unwrap();
    bibx(&server, dir.path())
        .args(["add", "eprint:hep-th/9711200", "--dry-run"])
        .assert()
        .success()
        .stdout(RECORD.trim_end().to_string() + "\n")
        .stderr("");
    assert_eq!(fs::read_to_string(&path).unwrap(), "@book{a}\n");
}

#[test]
fn duplicate_key_exits_3_and_leaves_the_file_alone() {
    let (server, dir) = (server(), tempfile::tempdir().unwrap());
    let path = dir.path().join("refs.bib");
    fs::write(&path, "@book{Maldacena:1997re}\n").unwrap();
    bibx(&server, dir.path())
        .args(["add", "eprint:hep-th/9711200"])
        .assert()
        .code(3)
        .stderr(predicates::str::contains("pass --key"));
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "@book{Maldacena:1997re}\n"
    );
}

#[test]
fn ambiguous_inspire_search_exits_4_and_leaves_the_file_alone() {
    let server = MockServer::start(vec![(
        "/api/literature?q=texkey%3Atest%3Akey&size=2&format=bibtex".into(),
        Reply::bibtex("@article{first}\n@article{second}\n"),
    )]);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("refs.bib");
    fs::write(&path, "@book{existing}\n").unwrap();
    bibx(&server, dir.path())
        .args(["add", "inspire:test:key"])
        .assert()
        .code(4)
        .stdout("")
        .stderr(predicates::str::contains("use a record id or an eprint"));
    assert_eq!(fs::read_to_string(&path).unwrap(), "@book{existing}\n");
}

#[test]
fn unknown_record_exits_1() {
    let (server, dir) = (server(), tempfile::tempdir().unwrap());
    fs::write(dir.path().join("refs.bib"), "").unwrap();
    bibx(&server, dir.path())
        .args(["add", "eprint:9999.99999"])
        .assert()
        .code(1)
        .stderr("error: no entry matches `eprint:9999.99999`\n");
}

#[test]
fn crlf_file_gets_a_crlf_entry() {
    let (server, dir) = (server(), tempfile::tempdir().unwrap());
    let path = dir.path().join("refs.bib");
    fs::write(&path, "@book{a}\r\n").unwrap();
    bibx(&server, dir.path())
        .args(["add", "eprint:hep-th/9711200"])
        .assert()
        .success();
    let out = fs::read_to_string(&path).unwrap();
    assert!(out.starts_with("@book{a}\r\n\r\n@article{Maldacena:1997re,\r\n"));
    assert!(!out.replace("\r\n", "").contains('\n'));
}

fn remove_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("bibx").unwrap();
    cmd.current_dir(dir).env_clear();
    cmd
}

#[test]
fn remove_rewrites_only_the_entry_and_keeps_crlf() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("refs.bib");
    fs::write(&path, "@a{a}\r\n\r\n@b{b,\r\n  x = 1\r\n}\r\n\r\n@c{c}\r\n").unwrap();
    remove_cmd(dir.path())
        .args(["remove", "b"])
        .assert()
        .success()
        .stdout("@b{b,\r\n  x = 1\r\n}\n")
        .stderr("removed b from refs.bib\n");
    assert_eq!(fs::read_to_string(&path).unwrap(), "@a{a}\r\n\r\n@c{c}\r\n");
}

#[test]
fn remove_dry_run_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("refs.bib");
    fs::write(&path, "@a{a}\n").unwrap();
    remove_cmd(dir.path())
        .args(["remove", "a", "--dry-run"])
        .assert()
        .success()
        .stdout("@a{a}\n");
    assert_eq!(fs::read_to_string(&path).unwrap(), "@a{a}\n");
}

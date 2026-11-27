//! End-to-end tests that need a mock server, which mdtest cannot start.

use std::fs;
use std::io::Write;
use std::path::Path;

use assert_cmd::Command;
use bibx_core::provider::testing::{MockServer, Reply};

const RECORD: &str = include_str!("fixtures/record.bib");
const ROUTE: &str = "/api/arxiv/hep-th/9711200?format=bibtex";
const ARXIV_ID: &str = "2501.01234v2";
const ARXIV_ROUTE: &str = "/src/2501.01234v2";

fn source(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut tar = tar::Builder::new(Vec::new());
    for &(path, bytes) in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        tar.append_data(&mut header, path, bytes).unwrap();
    }
    gzip(&tar.into_inner().unwrap())
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    enc.write_all(bytes).unwrap();
    enc.finish().unwrap()
}

fn source_reply(bytes: Vec<u8>) -> Reply {
    Reply::bytes(200, "application/gzip", bytes)
}

fn pull(server: &MockServer, dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("bibx").unwrap();
    cmd.current_dir(dir)
        .env_clear()
        .env("BIBX_ARXIV_TEST_URL", &server.url)
        .env("BIBX_ARXIV_TEST_CACHE", dir.join("cache"))
        .env("BIBX_FILE", dir.join("refs.bib"))
        .arg("pull");
    cmd
}

#[test]
fn pull_reuses_a_versioned_bundle_after_listing_its_members() {
    let bytes = b"% comment\r\n@misc{unchanged, note = {\xff}}";
    let bundle = source(&[("tex/z.bib", bytes), ("a.bib", b"other")]);
    let server = MockServer::start(vec![(ARXIV_ROUTE.into(), source_reply(bundle.clone()))]);
    let dir = tempfile::tempdir().unwrap();
    let local_bib = dir.path().join("refs.bib");
    fs::write(&local_bib, b"local bibliography").unwrap();
    pull(&server, dir.path())
        .arg("url:https://arxiv.org/pdf/2501.01234v2.pdf?download=1")
        .assert()
        .code(4)
        .stdout("")
        .stderr(
            "error: several .bib files found, pass --path to choose one:\n  a.bib\n  tex/z.bib\n",
        );
    pull(&server, dir.path())
        .args(["eprint:2501.01234v2", "--path", "./tex/z.bib"])
        .assert()
        .success()
        .stdout(bytes.as_slice())
        .stderr("");
    pull(&server, dir.path())
        .args([ARXIV_ID, "--path", "z.bib"])
        .assert()
        .code(1)
        .stdout("")
        .stderr("error: no .bib member matches z.bib\n");
    assert_eq!(server.requests().len(), 1);
    assert_eq!(
        fs::read(dir.path().join("cache/2501.01234v2.gz")).unwrap(),
        bundle
    );
    assert_eq!(fs::read(local_bib).unwrap(), b"local bibliography");
}

#[test]
fn pull_caches_legacy_versions_independently_and_checks_latest_each_time() {
    let server = MockServer::start(vec![
        (
            "/src/hep-th/9711200v2".into(),
            source_reply(source(&[("refs.bib", b"v2")])),
        ),
        (
            "/src/hep-th/9711200v3".into(),
            source_reply(source(&[("refs.bib", b"v3")])),
        ),
        (
            "/src/hep-th/9711200".into(),
            source_reply(source(&[("refs.bib", b"latest")])),
        ),
    ]);
    let dir = tempfile::tempdir().unwrap();
    for (id, bytes) in [
        ("arxiv:hep-th/9711200v2", "v2"),
        ("hep-th/9711200v3", "v3"),
        ("hep-th/9711200v2", "v2"),
        ("hep-th/9711200", "latest"),
        ("hep-th/9711200", "latest"),
    ] {
        pull(&server, dir.path())
            .arg(id)
            .assert()
            .success()
            .stdout(bytes)
            .stderr("");
    }
    assert_eq!(
        server
            .requests()
            .iter()
            .map(|r| r.target.as_str())
            .collect::<Vec<_>>(),
        [
            "/src/hep-th/9711200v2",
            "/src/hep-th/9711200v3",
            "/src/hep-th/9711200",
            "/src/hep-th/9711200"
        ]
    );
    assert_eq!(
        fs::read_dir(dir.path().join("cache/hep-th"))
            .unwrap()
            .count(),
        2
    );
}

#[test]
fn pull_reports_absent_bibliographies_and_unsupported_sources() {
    let samples = [
        (
            source(&[("main.bbl", b"compiled refs")]),
            ".bbl file but no .bib file",
        ),
        (source(&[("main.tex", b"document")]), "no .bib or .bbl file"),
        (gzip(br"\documentclass{article}"), "no .bib or .bbl file"),
        (
            gzip(b"<html>Unexpected response</html>"),
            "unsupported arXiv source format",
        ),
        (b"%PDF-1.7".to_vec(), "unsupported arXiv source format"),
    ];
    for (sample, expected) in samples {
        let server = MockServer::start(vec![(ARXIV_ROUTE.into(), source_reply(sample))]);
        let dir = tempfile::tempdir().unwrap();
        pull(&server, dir.path())
            .arg(ARXIV_ID)
            .assert()
            .code(1)
            .stdout("")
            .stderr(predicates::str::contains(expected));
    }
    let server = MockServer::start(vec![]);
    let dir = tempfile::tempdir().unwrap();
    pull(&server, dir.path())
        .arg(ARXIV_ID)
        .assert()
        .code(1)
        .stdout("")
        .stderr("error: source is unavailable for arXiv:2501.01234v2\n");
}

#[test]
fn pull_rejects_ambiguous_paths_and_bad_arguments() {
    let bundle = source(&[("refs.bib", b"a"), ("refs.bib", b"b")]);
    let server = MockServer::start(vec![(ARXIV_ROUTE.into(), source_reply(bundle))]);
    let dir = tempfile::tempdir().unwrap();
    for args in [
        vec!["doi:10.1/test"],
        vec![ARXIV_ID, "--path", "../refs.bib"],
    ] {
        pull(&server, dir.path())
            .args(args)
            .assert()
            .code(2)
            .stdout("");
    }
    assert!(server.requests().is_empty());
    pull(&server, dir.path())
        .args([ARXIV_ID, "--path", "refs.bib"])
        .assert()
        .code(1)
        .stdout("")
        .stderr("error: archive contains more than one .bib member at refs.bib\n");
}

#[test]
fn pull_repairs_corrupt_cache_entries_and_works_without_a_cache() {
    let bundle = source(&[("refs.bib", b"valid")]);
    let server = MockServer::start(vec![(ARXIV_ROUTE.into(), source_reply(bundle.clone()))]);
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache");
    let cache_file = cache.join("2501.01234v2.gz");
    fs::create_dir(&cache).unwrap();
    fs::write(&cache_file, b"corrupt").unwrap();
    pull(&server, dir.path())
        .arg(ARXIV_ID)
        .assert()
        .success()
        .stdout("valid")
        .stderr("");
    assert_eq!(fs::read(&cache_file).unwrap(), bundle);
    fs::remove_file(cache_file).unwrap();
    fs::remove_dir(&cache).unwrap();
    fs::write(&cache, b"unavailable").unwrap();
    pull(&server, dir.path())
        .arg(ARXIV_ID)
        .assert()
        .success()
        .stdout("valid")
        .stderr("");
    assert_eq!(server.requests().len(), 2);
}

#[test]
fn pull_leaves_no_cache_entry_or_stdout_on_failed_downloads() {
    let bundle = source(&[("refs.bib", b"bibliography")]);
    let declared_length = bundle.len() + 1;
    let mut corrupt = bundle.clone();
    corrupt.truncate(corrupt.len() - 4);
    for (reply, expected) in [
        (
            source_reply(bundle.clone()).with_declared_length(declared_length),
            "could not download",
        ),
        (source_reply(corrupt), "invalid arXiv source archive"),
        (
            source_reply(bundle).with_declared_length(100 * 1024 * 1024 + 1),
            "compressed source exceeds",
        ),
    ] {
        let server = MockServer::start(vec![(ARXIV_ROUTE.into(), reply)]);
        let dir = tempfile::tempdir().unwrap();
        pull(&server, dir.path())
            .arg(ARXIV_ID)
            .assert()
            .code(1)
            .stdout("")
            .stderr(predicates::str::contains(expected));
        let cache = dir.path().join("cache");
        assert!(!cache.exists() || fs::read_dir(cache).unwrap().count() == 0);
    }
}

fn server() -> MockServer {
    MockServer::start(vec![(ROUTE.into(), Reply::bibtex(RECORD))])
}

fn bibx(server: &MockServer, dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("bibx").unwrap();
    cmd.current_dir(dir)
        .env_clear()
        .env("BIBX_INSPIRE_TEST_URL", &server.url);
    cmd
}

#[test]
fn add_appends_and_prints_the_entry_with_an_auto_discovered_file() {
    let (server, dir) = (server(), tempfile::tempdir().unwrap());
    let path = dir.path().join("refs.bib");
    fs::write(&path, "@book{a, title = {A}}\n").unwrap();
    let entry = format!("{}\n", RECORD.trim_end());
    bibx(&server, dir.path())
        .args(["add", "eprint:hep-th/9711200"])
        .assert()
        .success()
        .stdout(entry.clone())
        .stderr("");
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        format!("@book{{a, title = {{A}}}}\n\n{entry}")
    );
}

#[test]
fn add_with_an_explicit_file_appends_and_prints_the_overridden_key() {
    let (server, dir) = (server(), tempfile::tempdir().unwrap());
    let path = dir.path().join("refs.bib");
    fs::write(&path, "").unwrap();
    let entry = RECORD.trim_end().replacen("Maldacena:1997re", "mine", 1) + "\n";
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
        .stdout(entry.clone())
        .stderr("");
    assert_eq!(fs::read_to_string(&path).unwrap(), entry);
}

#[test]
fn add_creates_a_missing_explicit_file() {
    let (server, dir) = (server(), tempfile::tempdir().unwrap());
    let entry = format!("{}\n", RECORD.trim_end());
    bibx(&server, dir.path())
        .args(["add", "-f", "new.bib", "eprint:hep-th/9711200"])
        .assert()
        .success()
        .stdout(entry.clone());
    assert_eq!(
        fs::read_to_string(dir.path().join("new.bib")).unwrap(),
        entry
    );
}

#[test]
fn add_creates_the_file_named_by_bibx_file() {
    let (server, dir) = (server(), tempfile::tempdir().unwrap());
    bibx(&server, dir.path())
        .env("BIBX_FILE", "new.bib")
        .args(["add", "eprint:hep-th/9711200"])
        .assert()
        .success();
    assert!(dir.path().join("new.bib").exists());
}

#[test]
fn failed_fetch_and_dry_run_create_nothing() {
    let (server, dir) = (server(), tempfile::tempdir().unwrap());
    bibx(&server, dir.path())
        .args(["add", "-f", "new.bib", "eprint:9999.99999"])
        .assert()
        .code(1);
    bibx(&server, dir.path())
        .args(["add", "-f", "new.bib", "eprint:hep-th/9711200", "--dry-run"])
        .assert()
        .success()
        .stderr("[dry run] Skipping write to new.bib\n");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn add_without_a_file_in_an_empty_directory_still_errors() {
    let (server, dir) = (server(), tempfile::tempdir().unwrap());
    bibx(&server, dir.path())
        .args(["add", "eprint:hep-th/9711200"])
        .assert()
        .code(1)
        .stderr("error: no .bib file found, pass -f\n");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
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
        .stderr("[dry run] Skipping write to refs.bib\n");
    assert_eq!(fs::read_to_string(&path).unwrap(), "@book{a}\n");
}

#[test]
fn add_from_an_arxiv_url_or_a_bare_arxiv_id() {
    for id in [
        "url:https://arxiv.org/pdf/hep-th/9711200v2.pdf",
        "https://arxiv.org/abs/hep-th/9711200",
        "hep-th/9711200v2",
    ] {
        let (server, dir) = (server(), tempfile::tempdir().unwrap());
        let path = dir.path().join("refs.bib");
        fs::write(&path, "").unwrap();
        bibx(&server, dir.path())
            .args(["add", id])
            .assert()
            .success();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            format!("{}\n", RECORD.trim_end()),
            "{id}"
        );
    }
}

#[test]
fn dry_run_from_inspire_literature_url() {
    let server = MockServer::start(vec![(
        "/api/literature/451647?format=bibtex".into(),
        Reply::bibtex(RECORD),
    )]);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("refs.bib");
    fs::write(&path, "@book{a}\n").unwrap();
    bibx(&server, dir.path())
        .args([
            "add",
            "https://inspirehep.net/literature/451647",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(format!("{}\n", RECORD.trim_end()));
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

#[test]
fn add_keeps_an_entry_saved_while_the_fetch_was_pending() {
    use std::io::Read;
    use std::net::TcpListener;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("refs.bib");
    fs::write(&path, "@book{a, title = {A}}\n").unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let edited = "@book{a, title = {A}}\n\n@book{b, title = {B}}\n";
    let server = {
        let path = path.clone();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            fs::write(&path, edited).unwrap();
            let body = RECORD.as_bytes();
            let head = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/x-bibtex\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(head.as_bytes()).unwrap();
            stream.write_all(body).unwrap();
        })
    };
    let entry = format!("{}\n", RECORD.trim_end());
    Command::cargo_bin("bibx")
        .unwrap()
        .current_dir(dir.path())
        .env_clear()
        .env("BIBX_INSPIRE_TEST_URL", url)
        .args(["add", "eprint:hep-th/9711200"])
        .assert()
        .success()
        .stdout(entry.clone());
    server.join().unwrap();
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        format!("{edited}\n{entry}")
    );
}

#[test]
fn add_does_not_recreate_a_discovered_file_deleted_during_the_fetch() {
    use std::io::Read;
    use std::net::TcpListener;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("refs.bib");
    fs::write(&path, "@book{a, title = {A}}\n").unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let server = {
        let path = path.clone();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            fs::remove_file(&path).unwrap();
            let body = RECORD.as_bytes();
            let head = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/x-bibtex\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(head.as_bytes()).unwrap();
            stream.write_all(body).unwrap();
        })
    };
    Command::cargo_bin("bibx")
        .unwrap()
        .current_dir(dir.path())
        .env_clear()
        .env("BIBX_INSPIRE_TEST_URL", url)
        .args(["add", "eprint:hep-th/9711200"])
        .assert()
        .failure()
        .stdout("");
    server.join().unwrap();
    assert!(!path.exists());
}

#[test]
fn same_paper_under_another_key_exits_3_unless_allowed() {
    let (server, dir) = (server(), tempfile::tempdir().unwrap());
    let path = dir.path().join("refs.bib");
    let existing = "@book{mine, eprint = {hep-th/9711200v1}}\n";
    fs::write(&path, existing).unwrap();
    bibx(&server, dir.path())
        .args(["add", "eprint:hep-th/9711200", "--key", "other"])
        .assert()
        .code(3)
        .stderr(predicates::str::contains("already in the file as `mine`"));
    assert_eq!(fs::read_to_string(&path).unwrap(), existing);
    bibx(&server, dir.path())
        .args([
            "add",
            "eprint:hep-th/9711200",
            "--key",
            "other",
            "--allow-duplicate",
        ])
        .assert()
        .success();
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .contains("@article{other,")
    );
}

#[test]
fn pull_output_writes_the_file_after_success() {
    let bytes = b"% comment\r\n@misc{unchanged, note = {\xff}}";
    let bundle = source(&[("refs.bib", bytes)]);
    let server = MockServer::start(vec![(ARXIV_ROUTE.into(), source_reply(bundle))]);
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.bib");
    pull(&server, dir.path())
        .args([ARXIV_ID, "-o"])
        .arg(&out)
        .assert()
        .success()
        .stdout("");
    assert_eq!(fs::read(&out).unwrap(), bytes);
}

#[test]
fn pull_output_leaves_an_existing_file_untouched_on_failure() {
    let server = MockServer::start(vec![]);
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.bib");
    fs::write(&out, b"precious").unwrap();
    pull(&server, dir.path())
        .args([ARXIV_ID, "--force", "-o"])
        .arg(&out)
        .assert()
        .code(1)
        .stdout("");
    assert_eq!(fs::read(&out).unwrap(), b"precious");
}

#[test]
fn pull_output_refuses_to_overwrite_without_force() {
    let bundle = source(&[("refs.bib", b"@misc{new}")]);
    let server = MockServer::start(vec![(ARXIV_ROUTE.into(), source_reply(bundle))]);
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.bib");
    fs::write(&out, b"precious").unwrap();
    pull(&server, dir.path())
        .args([ARXIV_ID, "-o"])
        .arg(&out)
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicates::str::contains("already exists"));
    assert_eq!(fs::read(&out).unwrap(), b"precious");
    assert!(server.requests().is_empty());
    pull(&server, dir.path())
        .args([ARXIV_ID, "--force", "-o"])
        .arg(&out)
        .assert()
        .success();
    assert_eq!(fs::read(&out).unwrap(), b"@misc{new}");
    pull(&server, dir.path())
        .args([ARXIV_ID, "--force"])
        .assert()
        .code(2);
}

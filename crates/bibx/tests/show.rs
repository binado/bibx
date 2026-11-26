use std::fs;

use assert_cmd::Command;

#[test]
fn show_prints_only_the_original_entry_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("refs.bib");
    let entry = "@ArTiClE(Example:2026,\r\n  title = {A fine example},\r\n  eprint = \"2611.12345v2\",\r\n  doi = {https://doi.org/10.1000/Example},\r\n)";
    let source = format!("% heading\r\n    {entry}  \r\n\r\n@book{{Next}}\r\n");
    fs::write(&path, &source).unwrap();

    Command::cargo_bin("bibx")
        .unwrap()
        .current_dir(dir.path())
        .env_clear()
        .args(["show", "Example:2026"])
        .assert()
        .success()
        .stdout(entry)
        .stderr("");

    fs::write(dir.path().join("other.bib"), "@book{Other}\n").unwrap();
    Command::cargo_bin("bibx")
        .unwrap()
        .current_dir(dir.path())
        .env_clear()
        .env("BIBX_FILE", "refs.bib")
        .args(["show", "arxiv:2611.12345"])
        .assert()
        .success()
        .stdout(entry)
        .stderr("");

    Command::cargo_bin("bibx")
        .unwrap()
        .current_dir(dir.path())
        .env_clear()
        .env("BIBX_FILE", "other.bib")
        .args(["show", "-f", "refs.bib", "doi:10.1000/example"])
        .assert()
        .success()
        .stdout(entry)
        .stderr("");

    assert_eq!(fs::read(path).unwrap(), source.as_bytes());
}

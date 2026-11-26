use std::fs;
use std::process::Command;

#[test]
fn checking_a_file_preserves_its_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("refs.bib");

    for (source, code) in [
        ("@book{réf}\r\n", 0),
        ("@book{réf}\r\n@book{réf}\r\n", 3),
        ("@book{réf}\r\n@book{bad, year = }", 1),
    ] {
        fs::write(&path, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_bibx"))
            .args(["check", "--file"])
            .arg(&path)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(code));
        assert_eq!(fs::read(&path).unwrap(), source.as_bytes());
    }
}

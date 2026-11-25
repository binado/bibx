use std::path::{Path, PathBuf};

/// Every `.bib` file under `tests/fixtures`, sorted.
pub fn fixtures() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "bib") {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"),
        &mut out,
    );
    out.sort();
    out
}

pub fn read(path: &Path) -> String {
    String::from_utf8(std::fs::read(path).unwrap())
        .unwrap_or_else(|_| panic!("{path:?} is not UTF-8"))
}

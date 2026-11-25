//! Markdown fixture tests: each `resources/mdtest/*.md` file is one libtest case whose
//! sections run `bibx` against inline input files and compare the output.

use std::path::Path;

fn bibx(path: &Path) -> datatest_stable::Result<()> {
    mdtest::run_file(
        path,
        &mdtest::Config {
            program: "bibx",
            bin: Path::new(env!("CARGO_BIN_EXE_bibx")),
        },
    )
}

datatest_stable::harness! {
    { test = bibx, root = "resources/mdtest", pattern = r"\.md$" },
}

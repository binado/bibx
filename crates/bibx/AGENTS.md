# AGENTS.md (bibx CLI)

The CLI: argv parsing, stdout/stderr formatting and exit codes. Keep it thin; logic lives in `bibx-core`. Data goes to stdout, messages to stderr.

Exit codes: 0 ok, 1 error, 2 usage, 3 duplicate key, 4 ambiguous match.

## Adding a CLI test

Prefer a Markdown fixture in `resources/mdtest/*.md` (run by `tests/mdtest.rs`; format in [`crates/mdtest/README.md`](../mdtest/README.md)). Write the console block, run `just bless`, review the diff. Tests that need the network mock server stay in `tests/cli.rs`.

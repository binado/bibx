# AGENTS.md

Repo-wide guidance for AI agents. Crate-specific rules live in `crates/<crate>/AGENTS.md`.

- Commits must follow Conventional Commits.
- Keep `README.md` examples and tables in sync with CLI changes. Keep the README and comments concise: detail belongs in `--help`, rustdoc and tests.
- Don't use code comments to enforce invariance or patterns. Those should be manifested in the code design.
- `IDEA.md` is the design document; update it when a decision changes.
- Never re-serialize the user's `.bib` file (only a future `fmt` may): edits are byte-range splices.

## Commands

MSRV is Rust 1.88.

```bash
mise run ci                                # format-check + lint + test + build + doc
mise run bless                             # rewrite expected output in Markdown fixtures (MDTEST_UPDATE=1)
cargo test -p bibx-parser                  # parity, recovery and round-trip tests
cargo test --workspace -- --ignored        # live INSPIRE tests (network)
```

## Crates

- `crates/bibx-parser/`: Lossless BibTeX parser with spans. No runtime dependencies.
- `crates/bibx-core/`: Model, identifiers, file discovery, providers, operations. No `clap`.
- `crates/bibx/`: The CLI (package and binary `bibx`). Thin.
- `crates/mdtest/`: Markdown fixture harness (`bibx-mdtest`, dev-only, not published).

# bibx-core

The library behind the [`bibx`](https://crates.io/crates/bibx) CLI: bibliography model, identifiers (`key`, `eprint`, `doi`, `inspire`), file discovery, INSPIRE and arXiv providers, and operations that return plans (add, remove) instead of writing files. Edits are byte-range splices of the original source, built on [`bibx-parser`](https://crates.io/crates/bibx-parser).

Providers sit behind the `inspire` and `arxiv` cargo features, both on by default.

The API is not stable: it exists to serve the CLI and may change in any release, so depend on it only with an exact version (`=x.y.z`). The `testing` feature (a mock HTTP server and `BIBX_*_TEST_URL` overrides) is for bibx's own tests and has no compatibility guarantee.

MSRV is Rust 1.88. Licensed under MIT.

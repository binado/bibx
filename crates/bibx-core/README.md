# bibx-core

The library behind the [`bibx`](https://crates.io/crates/bibx) CLI: bibliography model, identifiers (`key`, `eprint`, `doi`, `inspire`), file discovery, INSPIRE and arXiv providers, and operations that return plans (add, remove) instead of writing files. Edits are byte-range splices of the original source, built on [`bibx-parser`](https://crates.io/crates/bibx-parser).

Providers sit behind the `inspire` and `arxiv` cargo features, both on by default.

This crate follows the CLI's release cadence and its API may change between minor versions. See the [repository](https://github.com/binado/bibx) for details.

MSRV is Rust 1.88. Licensed under MIT.

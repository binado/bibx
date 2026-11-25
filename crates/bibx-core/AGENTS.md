# AGENTS.md (bibx-core)

Data model, identifiers, file discovery, providers and operations, built on `bibx-parser`. No `clap`, no printing: operations return plans (`AddPlan`, `RemovePlan`) and the CLI decides what to write.

- Edits are byte-range splices of the original source; never re-serialize the file.
- Providers live behind cargo features (`inspire`) and are tested against saved responses in `tests/fixtures/`.

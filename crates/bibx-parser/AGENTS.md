# AGENTS.md (bibx-parser)

Hand-written, lossless BibTeX parser that records byte spans. No runtime dependencies (`biblatex` and `proptest` are dev-only). Nothing here knows about the rest of bibx and nothing interprets LaTeX.

- Items are contiguous and cover the whole input; never index slices directly (use `get`).
- Only split at ASCII bytes so spans stay on char boundaries.
- Deliberate differences from `biblatex` are documented in the crate docs and tested in `tests/recovery.rs`; everything else must stay in parity (`tests/parity.rs`).

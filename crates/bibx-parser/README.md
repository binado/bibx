# bibx-parser

A lossless BibTeX parser that records byte spans. `parse` splits the input into contiguous items covering every byte, so concatenating them gives back the input exactly. Parsing never fails or panics: a malformed entry becomes an error item and parsing resumes at the next entry. No runtime dependencies.

```rust
let file = bibx_parser::parse("@article{key, title = {A title}}\n");
```

See the crate documentation for the data model and the deliberate differences from `biblatex`.

The API is not stable: it exists to serve bibx and may change in any release, so depend on it only with an exact version (`=x.y.z`).

MSRV is Rust 1.88. Licensed under MIT.

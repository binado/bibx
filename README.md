# bibx

A CLI that edits a BibTeX file in place with small diffs: fetch entries from INSPIRE, remove them and list them.

## Installation

```sh
cargo install --path crates/bibx
```

## Usage

```sh
bibx add arxiv:hep-th/9711200           # fetch from INSPIRE and append to the .bib file
bibx add doi:10.1103/PhysRevLett.116.061102 --key ligo --dry-run
bibx list --sort year                   # table; --format json for scripts
bibx remove Maldacena:1997re            # also eprint:<id> and doi:<doi>
```

The bib file is `-f/--file`, then `BIBX_FILE`, then the only `*.bib` in the current directory. Run `bibx <command> --help` for options and exit codes.

## Features

- Edits are byte-range splices: the rest of the file stays byte for byte, and line endings (LF or CRLF) are kept
- Identifiers of the form `<kind>:<value>` (key, eprint, doi, inspire), with `arxiv:` as an alias for `eprint:`
- A duplicate key or an ambiguous match is refused, with its own exit code
- Lossless parser with spans and per-entry error recovery (`bibx-parser`)

## Caveats

- To edit a bibliography reached through a symbolic link, select the target file directly
- INSPIRE is the only provider so far; arXiv, ADS, `check`, `fmt` and v2 commands are in the roadmap in `IDEA.md`
- Values are shown as raw LaTeX; nothing is decoded

## Acknowledgements

The mdtest harness is copied from [knf](https://github.com/binado/knf); `biblatex` is the parser's reference in tests, and some of its test files are reused (see `crates/bibx-parser/tests/fixtures/biblatex/ATTRIBUTION`).

## License

MIT, see [LICENSE](LICENSE).

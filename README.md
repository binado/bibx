# bibx

A CLI for inspecting, checking, and editing BibTeX files and extracting submitted bibliographies from arXiv.

## Installation

```sh
cargo install --path crates/bibx
```

## Usage

```sh
bibx add arxiv:hep-th/9711200           # fetch from INSPIRE, append to the .bib file, and print the entry
bibx add 'url:https://arxiv.org/pdf/hep-th/9711200v2.pdf'
bibx add 'url:https://inspirehep.net/literature/451647'
bibx add doi:10.1103/PhysRevLett.116.061102 --key ligo --dry-run  # preview and report the skipped write on stderr
bibx list --sort year                   # table; --format json for scripts
bibx show Maldacena:1997re              # original entry text, through its closing delimiter
bibx remove Maldacena:1997re            # also eprint:<id> and doi:<doi>
bibx check -f refs.bib                  # report syntax errors and duplicate keys, eprints and DOIs
bibx pull arxiv:2501.01234v2 > references.bib
bibx pull arxiv:2501.01234v2 --path tex/refs.bib > references.bib
```

For commands that work with a local bibliography, set the bib file with `-f/--file`, as in `bibx list -f refs.bib`. Without the flag, bibx uses `BIBX_FILE` if set. Leave it unset to discover the only `*.bib` in the current directory. Run `bibx <command> --help` for options and exit codes. `list`, `show`, `add` and `remove` ignore malformed entries and warn on stderr with the count; `bibx check` shows where.

`bibx check` leaves the file untouched and reports findings to stderr with file, line, and column. It exits with code 0 for a clean file, 3 for duplicates alone, 1 for syntax or file errors, and 2 for invalid usage.

## Features

- Edits are byte-range splices: the rest of the file stays byte for byte, and line endings (LF or CRLF) are kept
- Identifiers of the form `<kind>:<value>` (key, eprint, doi, inspire), with `arxiv:` as an alias for `eprint:`
- `add` accepts `url:` followed by an arXiv abstract or PDF URL or an INSPIRE literature record URL
- `pull` writes the original bibliography bytes to stdout and caches explicitly versioned source bundles in the platform user cache
- `add` refuses a duplicate key, or a paper whose eprint or DOI is already in the file (`--allow-duplicate` overrides the latter); an ambiguous match is refused too, each with its own exit code
- `add` fetches before reading the file and refuses to write if the file changed meanwhile
- Lossless parser with spans and per-entry error recovery (`bibx-parser`)

## Caveats

- To edit a bibliography reached through a symbolic link, select the target file directly
- `add` fetches individual entries from INSPIRE
- ADS and `fmt` are not implemented yet; see the [GitHub issues](https://github.com/binado/bibx/issues)
- `list` decodes LaTeX where it can, for the table and for `--sort`; JSON values stay raw (see `bibx list --help`)

## Development

With Rust and [mise](https://mise.jdx.dev/) installed, run `mise install` in each clone to install prek and register the Git pre-commit hook. Prek runs `bibx check` on staged bibliography files outside the test fixture directories. Run `mise run ci` for the full local checks or `mise tasks ls` to list tasks.

## Acknowledgements

The mdtest harness is copied from [knf](https://github.com/binado/knf); `biblatex` decodes LaTeX for `list` and is the parser's reference in tests, where some of its test files are reused (see `crates/bibx-parser/tests/fixtures/biblatex/ATTRIBUTION`).

## License

MIT, see [LICENSE](LICENSE).

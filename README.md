# bibx

A CLI that edits a BibTeX file in place with small diffs: fetch entries from INSPIRE, remove them and list them.

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
bibx remove Maldacena:1997re            # also eprint:<id> and doi:<doi>
```

Set the bib file with `-f/--file` after the subcommand, as in `bibx list -f refs.bib`. Without the flag, bibx uses `BIBX_FILE` if set. Leave it unset to discover the only `*.bib` in the current directory. Run `bibx <command> --help` for options and exit codes.

## Features

- Edits are byte-range splices: the rest of the file stays byte for byte, and line endings (LF or CRLF) are kept
- Identifiers of the form `<kind>:<value>` (key, eprint, doi, inspire), with `arxiv:` as an alias for `eprint:`
- `add` accepts `url:` followed by an arXiv abstract or PDF URL or an INSPIRE literature record URL
- A duplicate key or an ambiguous match is refused, with its own exit code
- Lossless parser with spans and per-entry error recovery (`bibx-parser`)

## Caveats

- To edit a bibliography reached through a symbolic link, select the target file directly
- INSPIRE is the only provider so far; arXiv, ADS, `check`, `fmt` and v2 commands are in the roadmap in `IDEA.md`
- `list` decodes LaTeX where it can, for the table and for `--sort`; JSON values stay raw (see `bibx list --help`)

## Development

With Rust and [mise](https://mise.jdx.dev/) installed, run `mise install` in each clone to install prek and register the Git pre-commit hook. Run `mise run ci` for the full local checks or `mise tasks ls` to list tasks.

## Acknowledgements

The mdtest harness is copied from [knf](https://github.com/binado/knf); `biblatex` decodes LaTeX for `list` and is the parser's reference in tests, where some of its test files are reused (see `crates/bibx-parser/tests/fixtures/biblatex/ATTRIBUTION`).

## License

MIT, see [LICENSE](LICENSE).

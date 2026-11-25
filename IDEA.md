# bibx

Pitch: a CLI for managing a bib file. Name follows `npx`/`uvx`/`pipx`: tools that fetch things from the network.

Name: `bibx` for both the crate and the command (free on crates.io and Homebrew, checked 2026-11-25).
`bib` is taken on crates.io (an abandoned Bitbucket CLI), and `bibr` is too close to `biber`
(the biblatex backend).

## Usage

The subcommand comes first. The bib file is an optional `-f/--file` flag:

```sh
bibx <COMMAND> [-f <FILE>] [ARGS]
```

### Choosing the bib file

1. If `-f/--file` is given, use it.
2. Otherwise, if the `BIBX_FILE` env var is set, use it (e.g. set per project with direnv).
3. Otherwise, look for `*.bib` files in the current directory (not recursive):
   - exactly one: use it
   - none: fail with "no .bib file found, pass -f"
   - more than one: fail and list them, asking for `-f`

Notes:

- `-f` is a global clap arg (`global = true`), so `bibx add -f refs.bib ...` and
  `bibx -f refs.bib add ...` both work.
- `check` and `fmt` accept `-f` more than once to act on several files.
- Output names the file whenever it was auto-discovered
  (e.g. `added Author:2020abc to refs.bib`), so a wrong guess is visible.

### v1

```sh
# add: fetch an entry from a provider and append it to the file
bibx add eprint:2005.12345                           # source defaults to inspire
bibx add eprint:2005.12345 --source arxiv
bibx add doi:10.1103/PhysRevD.102.123456 --key mypaper
bibx add inspire:Author:2020abc -f refs.bib
bibx add ads:2020ApJ...900..123A --source ads

# remove: delete one entry
bibx remove Author:2020abc                           # bare value = citation key
bibx remove eprint:2005.12345
bibx remove doi:10.1103/PhysRevD.102.123456

# list
bibx list --sort year
bibx list --fields title,year
bibx list --format json > bib.json
```

### Later

```sh
bibx check                     # check the file is valid and has no duplicates
bibx fmt -f a.bib -f b.bib     # the only command allowed to rewrite a whole file

# v2
bibx add eprint:2005.12345 doi:10.1103/...  # several ids at once (also from stdin)
bibx update                    # replace arXiv-only entries with their published versions (INSPIRE/ADS)
bibx sync paper.tex            # fetch every \cite key in paper.tex that is missing from the bib file
```

## Identifiers

Identifiers have the form `<kind>:<value>`, which is generic and easy to extend:

| Kind      | Example                           | Matches field / used for       |
|-----------|-----------------------------------|--------------------------------|
| `key`     | `key:Author:2020abc`              | citation key                   |
| `eprint`  | `eprint:2005.12345`               | `eprint` field (arXiv id)      |
| `doi`     | `doi:10.1103/PhysRevD.102.123456` | `doi` field                    |
| `inspire` | `inspire:Author:2020abc`          | INSPIRE texkey / record id     |
| `ads`     | `ads:2020ApJ...900..123A`         | ADS bibcode                    |

Parsing rules:

- Split on the **first** colon only (DOIs and texkeys can contain colons).
- Only a **known** prefix counts as a kind. Anything else is a bare citation key, so
  `Author:2020abc` is a key, not kind `Author`. `key:` can be written explicitly when a
  key begins with a known prefix.
- For `remove` (and other lookups), a bare value defaults to a citation key.
- If a lookup matches more than one entry, fail and list the matches instead of guessing.

## Sources (providers)

The identifier kind says *what* the paper is; `--source` says *where* to fetch it from.
They are separate, so `eprint:2005.12345` can be fetched from INSPIRE, arXiv or ADS.

- **v1 ships INSPIRE only.** It is the default source: it accepts eprints, DOIs and texkeys,
  and returns good BibTeX keys (texkeys), so v1 needs no key generation.
- Later, `arxiv`: the API returns Atom XML, so we build the BibTeX entry and generate a key
  ourselves (INSPIRE-like `Lastname:YYYYabc` or simpler `Lastname2020`).
- Later, `ads`: needs an API token, read from the `ADS_API_TOKEN` env var (no config file
  until a second setting needs one).
- `--key` overrides the citation key the provider would choose.

### INSPIRE API notes

Source: https://github.com/inspirehep/rest-api-doc (checked 2026-11-25).

| Identifier | Request | Result |
|---|---|---|
| `eprint:` | `GET /api/arxiv/<id>?format=bibtex` | single record; old-style ids (`hep-th/9711200`) work |
| `doi:` | `GET /api/doi/<doi>?format=bibtex` | single record |
| `inspire:` (record id) | `GET /api/literature/<recid>?format=bibtex` | single record |
| `inspire:` (texkey) | `GET /api/literature?q=texkey:<key>&format=bibtex` | **a search**: check there is exactly one hit |

- Base URL `https://inspirehep.net`. Success is `200` with `Content-Type: application/x-bibtex`.
- The BibTeX key returned is the INSPIRE texkey (e.g. `Maldacena:1997re`).
- Missing record: `404` with a JSON body (`{"message": "...PID does not exist..."}`). Map to `NotFound`.
- Versioned arXiv ids (`1706.03762v5`) return 404, so strip `vN` before querying.
- `inspire:` values: all digits = record id, otherwise texkey.
- Rate limit: 15 requests per IP per 5 s. Exceeding gives `429` (no `Retry-After` documented);
  wait at least 5 s. Irrelevant for a single `add`, but batch `add` and `sync` (v2) must throttle.
- Usage is governed by INSPIRE's terms of use; no User-Agent requirement is documented. Send
  a generic one like `bibx/<version>`.

The provider trait returns a structured entry (plus the raw BibTeX when the source gives one),
so sources that don't send BibTeX fit too.

## Editing rules (keeping diffs small)

The user's file is never re-serialized, except by `bibx fmt`.

- `add` **appends** the new entry's text to the end of the file. Nothing else changes.
- Before appending, `add` ensures the file ends with a newline, then adds a blank line and the entry.
- `add` appends the provider's BibTeX exactly as received; with `--key`, only the key in the
  entry header is rewritten. Restyling is left to `fmt`.
- v1 duplicate check is **key collisions only**: if the citation key already exists, `add` refuses
  and suggests `--key`. Never add a suffix silently. Content-based duplicates (same eprint/doi
  under another key, with arXiv/DOI normalization) come later.
- `remove` deletes only the text range of the matched entry and writes the result to a temp file,
  then renames it over the original (no half-written files).
- Keep the file's line endings (LF or CRLF). Assume UTF-8 and fail on anything else.
- `--dry-run` on `add` and `remove` shows what would change without writing. `remove` always
  prints the entry it removed.
- So the parser must record each entry's position in the file (see [Parser](#parser)).
- Test: parsing a file and writing it back unchanged must give back the identical bytes.

## Design decisions

- Cargo workspace with three crates:
  - `bibx-parser`: our own hand-written BibTeX parser, no dependency on the rest of bibx
    (see [Parser](#parser)).
  - `bibx-core`: data model, identifier parsing, the `Provider` trait and operations, built on
    `bibx-parser`. Providers (`inspire`, `arxiv`, `ads`) are modules behind cargo features.
  - `bibx-cli`: clap wrapper over core. One Rust file per subcommand.
    - The subcommand comes first. The top-level `Cli` holds the subcommand and a global
      `-f/--file: Vec<PathBuf>`. Finding the file (flag, then `BIBX_FILE`, then auto-discovery) is a single
      function in core that every subcommand calls; it is easy to test with a temp dir.
    - `--fields` uses `value_delimiter = ','`.
- Split more crates out (e.g. a provider) only when something needs them on their own.
- HTTP: a blocking client (`ureq` or `reqwest::blocking`) for v1, with a timeout and a generic
  User-Agent. Revisit when adding several ids at once.
- Errors: structured error types in core (`thiserror`: `DuplicateKey`, `Ambiguous`, `NotFound`,
  `Network`, `Parse`, ...), mapped to good human messages in the CLI (`anyhow` at the top).
  Duplicate and ambiguous matches get their own exit codes so scripts can tell them apart.
- Output: data to stdout, messages to stderr (`bibx list --format json | jq` works).
  `list` defaults to a compact table. The JSON field names are a stable interface once released.
- Testing: providers are tested against saved API responses. A few live tests run only on
  request (`#[ignore]`).

## Parser

`bibx-parser` is hand-written and kept small: it finds where everything is, it does not interpret LaTeX.
Turning LaTeX into readable text (`{\"o}` -> `ö`, name lists, `@string` expansion, `crossref`)
is out of scope for v1.

### What it does

- **Lossless:** the file is split into a list of items whose spans cover every byte, in order.
  Joining the items' source text gives back the input exactly, by construction.
- Items: `Entry`, `String` (`@string`), `Preamble`, `Comment` (`@comment` and any text
  outside entries, which BibTeX treats as a comment) and `Error` (a malformed entry).
- For each entry: type, key and fields, each with its span. Fields stay in source order with raw
  values (braces and LaTeX intact).
- Syntax covered: `{...}` and `(...)` entry delimiters; values in nested braces, in quotes
  (braces inside quotes count), bare numbers or `@string` names, joined with `#`; `@comment`
  with nested braces.
- **Error recovery:** a malformed entry becomes an `Error` item (span + message), and parsing
  resumes at the next `@` that starts a line. One broken entry doesn't hide the rest.
- Never panics on any input.

Sketch of the types (borrowing from the source `&str`):

```rust
pub struct Bib<'s> { pub items: Vec<Item<'s>> }

pub enum Item<'s> {
    Entry(Entry<'s>),
    String(Entry<'s>),        // same shape: fields are the definitions
    Preamble(Span),
    Comment(Span),
    Error { span: Span, error: ParseError },
}

pub struct Entry<'s> {
    pub span: Span,           // from '@' to the closing delimiter, inclusive
    pub kind: Spanned<&'s str>,
    pub key: Spanned<&'s str>,
    pub fields: Vec<Field<'s>>,
}

pub struct Field<'s> {
    pub span: Span,
    pub name: Spanned<&'s str>,
    pub value: Vec<Spanned<ValuePart<'s>>>,  // parts joined by '#'
}

pub enum ValuePart<'s> { Braced(&'s str), Quoted(&'s str), Number(&'s str), Macro(&'s str) }
```

### Tests

- **Parity with `biblatex`:** `biblatex` (raw layer, `RawBibliography::parse`) is a
  dev-dependency of `bibx-parser`, used as the reference in tests and never shipped.
  - Corpus in `bibx-parser/tests/fixtures/`: real INSPIRE/ADS exports, old arXiv-era files,
    and edge cases (nested braces, quotes, `#` concatenation, `@string`, parentheses, CRLF,
    UTF-8). `biblatex`'s own test files can be reused too (MIT/Apache-2.0, keep attribution).
  - For every fixture `biblatex` accepts: both parsers find the same entries in the same order
    with the same type, key, field names and raw field values.
  - Normalize before comparing where the two are known to differ, e.g. `biblatex`'s entry span
    stops before the closing `}` and its macro values include trailing whitespace. Field name
    case may need normalizing too: check it.
  - Where we deliberately differ (nested braces in `@comment`, error recovery), write explicit
    tests instead of parity tests, and list the differences in the crate docs.
  - For fixtures `biblatex` rejects: we must not panic, must round-trip, and must report an
    `Error` item at the right place.
- **Round-trip property test** (`proptest`): for arbitrary strings, not just valid BibTeX,
  joining the items' source text gives back the input.
- Optional: `cargo fuzz` target to catch panics.

## Roadmap

1. `bibx-parser` + parity tests against `biblatex` + round-trip property test
2. `list`
3. `add` via INSPIRE, with duplicate checks
4. `remove`
5. arXiv and ADS providers, content-based duplicate detection
6. `check`, `fmt`
7. v2: several ids per `add`, `update`, `sync`

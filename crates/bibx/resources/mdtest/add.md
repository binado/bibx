# Add (no network)

`refs.bib`:

```bibtex
@book{a, title = {A}}
```

## A bare key cannot be fetched

```console
$ bibx add Maldacena:1997re
? 1
```

`stderr`:

```text
error: cannot fetch the bare key `Maldacena:1997re`; use inspire:Maldacena:1997re
```

## Explicit key prefix is also a key

```console
$ bibx add key:Maldacena:1997re
? 1
```

`stderr`:

```text
error: cannot fetch the bare key `Maldacena:1997re`; use inspire:Maldacena:1997re
```

## Add takes a single file

`other.bib`:

```bibtex
```

```console
$ bibx add -f refs.bib -f other.bib eprint:2005.12345
? 2
```

`stderr`:

```text
error: this command takes a single -f/--file

Usage: bibx [OPTIONS] <COMMAND>

For more information, try '--help'.
```

## Unknown source is a usage error

```console
$ bibx add --source arxiv eprint:2005.12345
? 2
```

`stderr`:

```text
error: invalid value 'arxiv' for '--source <SOURCE>'
  [possible values: inspire]

For more information, try '--help'.
```

## Ads ids are not fetchable from INSPIRE

```console
$ BIBX_INSPIRE_URL=http://127.0.0.1:9 bibx add ads:2020ApJ...900..123A
? 1
```

`stderr`:

```text
error: INSPIRE cannot look up `ads` identifiers; use eprint:, doi: or inspire:
```

# Errors

## No bib file

```console
$ bibx list
? 1
```

`stderr`:

```text
error: no .bib file found, pass -f
```

## Unknown subcommand is a usage error

```console
$ bibx frobnicate
? 2
```

`stderr`:

```text
error: unrecognized subcommand 'frobnicate'

Usage: bibx <COMMAND>

For more information, try '--help'.
```

# File discovery

`a.bib`:

```bibtex
@article{A, title = {From a}}
```

## A flag before the subcommand is a usage error

```console
$ bibx -f a.bib list --fields key,title
? 2
```

`stderr`:

```text
error: unexpected argument '-f' found

Usage: bibx <COMMAND>

For more information, try '--help'.
```

## Flags on both sides of the subcommand are a usage error

```console
$ bibx -f a.bib list --file b.bib
? 2
```

`stderr`:

```text
error: unexpected argument '-f' found

Usage: bibx <COMMAND>

For more information, try '--help'.
```

## Repeating either spelling after the subcommand is a usage error

```console
$ bibx list --file a.bib -f b.bib
? 2
```

`stderr`:

```text
error: the argument '--file <FILE>' cannot be used multiple times

Usage: bibx list [OPTIONS]

For more information, try '--help'.
```

## Flag after the subcommand

```console
$ bibx list -f a.bib --fields key,title
```

`stdout`:

```text
┌─────┬────────┐
│ key ┆ title  │
╞═════╪════════╡
│ A   ┆ From a │
└─────┴────────┘
```

## One file in the current directory is found

```console
$ bibx list --fields key,title
```

`stdout`:

```text
┌─────┬────────┐
│ key ┆ title  │
╞═════╪════════╡
│ A   ┆ From a │
└─────┴────────┘
```

## Empty BIBX_FILE is a usage error

```console
$ BIBX_FILE= bibx list --fields key,title
? 2
```

`stderr`:

```text
error: a value is required for '--file <FILE>' but none was supplied

For more information, try '--help'.
```

## BIBX_FILE beats discovery

`b.bib`:

```bibtex
@article{B, title = {From b}}
```

```console
$ BIBX_FILE=b.bib bibx list --fields key,title
```

`stdout`:

```text
┌─────┬────────┐
│ key ┆ title  │
╞═════╪════════╡
│ B   ┆ From b │
└─────┴────────┘
```

## A flag beats BIBX_FILE

```console
$ BIBX_FILE=b.bib bibx list -f a.bib --fields key,title
```

`stdout`:

```text
┌─────┬────────┐
│ key ┆ title  │
╞═════╪════════╡
│ A   ┆ From a │
└─────┴────────┘
```

## Several files are an error

`b.bib`:

```bibtex
@article{B, title = {From b}}
```

```console
$ bibx list
? 1
```

`stderr`:

```text
error: several .bib files found, pass -f to choose one:
  a.bib
  b.bib
```

## An env file that does not exist

```console
$ BIBX_FILE=missing.bib bibx list
? 1
```

`stderr`:

```text
error: missing.bib: No such file or directory (os error 2)
```

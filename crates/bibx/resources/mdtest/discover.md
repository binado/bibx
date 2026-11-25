# File discovery

`a.bib`:

```bibtex
@article{A, title = {From a}}
```

## Flag before the subcommand

```console
$ bibx -f a.bib list --fields key,title
```

`stdout`:

```text
┌─────┬────────┐
│ key ┆ title  │
╞═════╪════════╡
│ A   ┆ From a │
└─────┴────────┘
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

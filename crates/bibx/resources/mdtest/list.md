# List

`refs.bib`:

```bibtex
@article{Zeta:2001a,
  title = {Zeta
           function},
  author = {Doe, J.},
  year = 2001
}

@book{Alpha:1999,
  title = "Alpha",
  year = {1999},
}

@misc{NoYear,
  title = {No year here}
}

@string{foo = "bar"}
% a comment
```

## Default columns

```console
$ bibx list
```

`stdout`:

```text
┌────────────┬──────┬───────────────┐
│ key        ┆ year ┆ title         │
╞════════════╪══════╪═══════════════╡
│ Zeta:2001a ┆ 2001 ┆ Zeta function │
│ Alpha:1999 ┆ 1999 ┆ Alpha         │
│ NoYear     ┆      ┆ No year here  │
└────────────┴──────┴───────────────┘
```

## Sort by year

```console
$ bibx list --sort year
```

`stdout`:

```text
┌────────────┬──────┬───────────────┐
│ key        ┆ year ┆ title         │
╞════════════╪══════╪═══════════════╡
│ Alpha:1999 ┆ 1999 ┆ Alpha         │
│ Zeta:2001a ┆ 2001 ┆ Zeta function │
│ NoYear     ┆      ┆ No year here  │
└────────────┴──────┴───────────────┘
```

## Sort by key

```console
$ bibx list --sort key --fields key,type
```

`stdout`:

```text
┌────────────┬─────────┐
│ key        ┆ type    │
╞════════════╪═════════╡
│ Alpha:1999 ┆ book    │
│ NoYear     ┆ misc    │
│ Zeta:2001a ┆ article │
└────────────┴─────────┘
```

## Fields are case-insensitive

```console
$ bibx list --fields TITLE,Year --sort title
```

`stdout`:

```text
┌───────────────┬──────┐
│ title         ┆ year │
╞═══════════════╪══════╡
│ Alpha         ┆ 1999 │
│ No year here  ┆      │
│ Zeta function ┆ 2001 │
└───────────────┴──────┘
```

## JSON

```console
$ bibx list --format json --fields year,author
```

`stdout`:

```text
[
  {
    "key": "Zeta:2001a",
    "type": "article",
    "fields": {
      "year": "2001",
      "author": "Doe, J."
    }
  },
  {
    "key": "Alpha:1999",
    "type": "book",
    "fields": {
      "year": "1999"
    }
  },
  {
    "key": "NoYear",
    "type": "misc",
    "fields": {}
  }
]
```

## Explicit file

```console
$ bibx list -f refs.bib --fields key
```

`stdout`:

```text
┌────────────┐
│ key        │
╞════════════╡
│ Zeta:2001a │
│ Alpha:1999 │
│ NoYear     │
└────────────┘
```

## Empty files

`empty.bib`:

```bibtex
```

### Table prints nothing

```console
$ bibx list -f empty.bib
```

### JSON is an empty array

```console
$ bibx list -f empty.bib --format json
```

`stdout`:

```text
[]
```

## LaTeX

`latex.bib`:

```bibtex
@article{Math,
  title = {Running of $\alpha_s$},
  pages = {231--252},
  doi = {10.1000/a--b_c}
}

@article{Braced,
  title = {{The {LHC}} \& 5\%},
  author = {Schr{\"o}dinger, E.}
}

@misc{Broken,
  title = {Cost $5 {today}}
}

@misc{Plain,
  title = {Zeta}
}
```

### The table is decoded where possible

```console
$ bibx list -f latex.bib --sort title --fields key,title,author,pages,doi
```

`stdout`:

```text
┌────────┬───────────────────────┬─────────────────┬─────────┬────────────────┐
│ key    ┆ title                 ┆ author          ┆ pages   ┆ doi            │
╞════════╪═══════════════════════╪═════════════════╪═════════╪════════════════╡
│ Broken ┆ Cost $5 {today}       ┆                 ┆         ┆                │
│ Math   ┆ Running of $\alpha_s$ ┆                 ┆ 231–252 ┆ 10.1000/a--b_c │
│ Braced ┆ The LHC & 5%          ┆ Schrödinger, E. ┆         ┆                │
│ Plain  ┆ Zeta                  ┆                 ┆         ┆                │
└────────┴───────────────────────┴─────────────────┴─────────┴────────────────┘
```

### JSON values stay raw but follow the decoded order

```console
$ bibx list -f latex.bib --sort title --format json
```

`stdout`:

```text
[
  {
    "key": "Broken",
    "type": "misc",
    "fields": {
      "title": "Cost $5 {today}"
    }
  },
  {
    "key": "Math",
    "type": "article",
    "fields": {
      "title": "Running of $\\alpha_s$"
    }
  },
  {
    "key": "Braced",
    "type": "article",
    "fields": {
      "title": "{The {LHC}} \\& 5\\%"
    }
  },
  {
    "key": "Plain",
    "type": "misc",
    "fields": {
      "title": "Zeta"
    }
  }
]
```

# Check

## A clean file is silent

`refs.bib`:

```bibtex
@string{a = "Journal"}
@preamble{"a"}
@comment{a}
@book{a, journal = a}
@book{A}
```

```console
$ bibx check
```

## Recover from multiple syntax errors

`syntax.bib`:

```bibtex
@article{bad1,
  title {Missing equals}
}
@book{good}
@article{bad2,
  year = }
```

```console
$ BIBX_FILE=syntax.bib bibx check
? 1
```

`stderr`:

```text
syntax.bib:2:9: error: expected `=` after the field name
syntax.bib:6:10: error: expected a field value
```

## Every repeated key points back to its first occurrence

`refs.bib`:

```bibtex
@book{repeated}
@book{other}
  @article{repeated}
@a(repeated)
```

```console
$ BIBX_FILE=ignored.bib bibx check -f refs.bib
? 3
```

`stderr`:

```text
refs.bib:3:12: error: duplicate citation key `repeated` (first occurrence at refs.bib:1:7)
refs.bib:4:4: error: duplicate citation key `repeated` (first occurrence at refs.bib:1:7)
```

## Syntax errors and duplicate keys appear together in source order

`mixed.bib`:

```bibtex
@book{a}
@book{missing, title {x}}
@book{a}
@book{a, year = }
@book{a}
```

```console
$ bibx check --file=mixed.bib
? 1
```

`stderr`:

```text
mixed.bib:2:22: error: expected `=` after the field name
mixed.bib:3:7: error: duplicate citation key `a` (first occurrence at mixed.bib:1:7)
mixed.bib:4:17: error: expected a field value
mixed.bib:5:7: error: duplicate citation key `a` (first occurrence at mixed.bib:1:7)
```

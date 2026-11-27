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

## A paper repeated under another key is reported

`papers.bib`:

```bibtex
@article{first, eprint = "1207.7214v2", doi = "10.1016/J.physletb.2012.08.020"}
@article{second, doi = {https://doi.org/10.1016/j.physletb.2012.08.020}}
@article{third, eprint = {arXiv:1207.7214}}
@article{third, eprint = {1207.7214}}
```

```console
$ bibx check -f papers.bib
? 3
```

`stderr`:

```text
papers.bib:2:10: error: `doi:10.1016/j.physletb.2012.08.020` repeats `first` (first occurrence at papers.bib:1:10)
papers.bib:3:10: error: `eprint:1207.7214` repeats `first` (first occurrence at papers.bib:1:10)
papers.bib:4:10: error: `eprint:1207.7214` repeats `first` (first occurrence at papers.bib:1:10)
papers.bib:4:10: error: duplicate citation key `third` (first occurrence at papers.bib:3:10)
```

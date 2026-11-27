# Remove

`refs.bib`:

```bibtex
@article{First:2001a,
  title = {First},
  eprint = "2001.00001v2",
  doi = "10.1000/First"
}

@article{Second:2002b,
  title = {Second},
  eprint = "2002.00002",
  doi = {https://doi.org/10.1000/Second}
}

@article{Third:2003c,
  title = {Third},
  eprint = "2003.00003"
}
```

## By key

```console
$ bibx remove Second:2002b
```

`stdout`:

```text
@article{Second:2002b,
  title = {Second},
  eprint = "2002.00002",
  doi = {https://doi.org/10.1000/Second}
}
```

`stderr`:

```text
removed Second:2002b from refs.bib
```

## First entry

```console
$ bibx remove First:2001a
```

`stdout`:

```text
@article{First:2001a,
  title = {First},
  eprint = "2001.00001v2",
  doi = "10.1000/First"
}
```

`stderr`:

```text
removed First:2001a from refs.bib
```

## Last entry

```console
$ bibx remove Third:2003c
```

`stdout`:

```text
@article{Third:2003c,
  title = {Third},
  eprint = "2003.00003"
}
```

`stderr`:

```text
removed Third:2003c from refs.bib
```

## By eprint using the arXiv alias, ignoring the version

```console
$ bibx remove arXiv:2001.00001v3
```

`stdout`:

```text
@article{First:2001a,
  title = {First},
  eprint = "2001.00001v2",
  doi = "10.1000/First"
}
```

`stderr`:

```text
removed First:2001a from refs.bib
```

## By DOI, ignoring case and resolver URL

```console
$ bibx remove doi:10.1000/second
```

`stdout`:

```text
@article{Second:2002b,
  title = {Second},
  eprint = "2002.00002",
  doi = {https://doi.org/10.1000/Second}
}
```

`stderr`:

```text
removed Second:2002b from refs.bib
```

## Dry run leaves the file alone

```console
$ bibx remove Second:2002b --dry-run
```

`stdout`:

```text
@article{Second:2002b,
  title = {Second},
  eprint = "2002.00002",
  doi = {https://doi.org/10.1000/Second}
}
```

`stderr`:

```text
would remove Second:2002b from refs.bib
```

## An explicit file is not named in the message

```console
$ bibx remove Second:2002b -f refs.bib
```

`stdout`:

```text
@article{Second:2002b,
  title = {Second},
  eprint = "2002.00002",
  doi = {https://doi.org/10.1000/Second}
}
```

`stderr`:

```text
removed Second:2002b
```

## Not found

```console
$ bibx remove Nope:1999
? 1
```

`stderr`:

```text
error: no entry matches `Nope:1999`
```

## Ambiguous

`dup.bib`:

```bibtex
@article{One, eprint = "1234.5678"}
@article{Two, eprint = "1234.5678v3"}
```

```console
$ bibx remove -f dup.bib eprint:1234.5678
? 4
```

`stderr`:

```text
error: `eprint:1234.5678` matches several entries: One, Two
```

## Malformed entries are skipped with a warning on stderr

`broken.bib`:

```bibtex
@article{A, title={Unclosed,
}
@book{B, title={ok}}
```

```console
$ bibx remove B -f broken.bib --dry-run
```

`stdout`:

```text
@book{B, title={ok}}
```

`stderr`:

```text
warning: skipped 1 malformed entry in broken.bib; run `bibx check` for details
would remove B
```

## URLs

```console
$ bibx remove --dry-run https://arxiv.org/abs/2002.00002v1
```

`stdout`:

```text
@article{Second:2002b,
  title = {Second},
  eprint = "2002.00002",
  doi = {https://doi.org/10.1000/Second}
}
```

`stderr`:

```text
would remove Second:2002b from refs.bib
```

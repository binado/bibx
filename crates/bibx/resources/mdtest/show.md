# Show

`refs.bib`:

```bibtex
@article{One, eprint = "1234.5678"}
@article{Two, eprint = "1234.5678v3"}
```

## Not found

```console
$ bibx show Missing
? 1
```

`stderr`:

```text
error: no entry matches `Missing`
```

## Ambiguous

```console
$ bibx show eprint:1234.5678
? 4
```

`stderr`:

```text
error: `eprint:1234.5678` matches several entries: One, Two
```

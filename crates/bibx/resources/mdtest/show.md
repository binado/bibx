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

## A URL matches by eprint

```console
$ bibx show https://arxiv.org/abs/1234.5678v2
? 4
```

`stderr`:

```text
error: `eprint:1234.5678` matches several entries: One, Two
```

## An unsupported URL is rejected

```console
$ bibx show https://example.com/abs/1234.5678
? 1
```

`stderr`:

```text
error: expected an arXiv or INSPIRE URL: https://arxiv.org/abs/<id>, https://arxiv.org/pdf/<id> or https://inspirehep.net/literature/<record-id>
```

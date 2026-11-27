# bibx

Inspect, check and edit BibTeX files from the command line, and pull submitted bibliographies from arXiv. Edits are byte-range splices, so the rest of your `.bib` file stays byte for byte.

```sh
bibx add arxiv:hep-th/9711200     # fetch from INSPIRE, append, and print the entry
bibx list --sort year             # table; --format json for scripts
bibx show Maldacena:1997re        # original entry text
bibx remove Maldacena:1997re
bibx check -f refs.bib            # syntax errors and duplicate keys
bibx pull arxiv:2501.01234v2 > references.bib
```

Run `bibx <command> --help` for options and exit codes. See the [repository](https://github.com/binado/bibx) for details.

MSRV is Rust 1.88. Licensed under MIT.

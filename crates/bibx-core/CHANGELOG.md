# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/binado/bibx/releases/tag/bibx-core-v0.1.0) - 2026-11-27

### Added

- *(pull)* add -o/--output ([#53](https://github.com/binado/bibx/pull/53))
- *(check)* report citation keys that differ only in case ([#63](https://github.com/binado/bibx/pull/63))
- *(add)* create the bibliography file when it does not exist ([#62](https://github.com/binado/bibx/pull/62))
- *(cli)* accept URLs and bare arXiv ids consistently ([#61](https://github.com/binado/bibx/pull/61))
- *(core)* gate the INSPIRE URL override behind the testing feature ([#60](https://github.com/binado/bibx/pull/60))
- *(add)* refuse a paper already in the file under another key ([#57](https://github.com/binado/bibx/pull/57))
- *(cli)* pull bibliographies from arXiv source ([#38](https://github.com/binado/bibx/pull/38))
- *(cli)* add bibx check and prek hook ([#37](https://github.com/binado/bibx/pull/37))
- *(cli)* show a bibliography entry verbatim ([#36](https://github.com/binado/bibx/pull/36))
- *(list)* warn when no entry has a requested field ([#30](https://github.com/binado/bibx/pull/30))
- *(list)* decode LaTeX in table output and sorting ([#20](https://github.com/binado/bibx/pull/20))
- *(cli)* accept arXiv and INSPIRE URLs in bibx add ([#18](https://github.com/binado/bibx/pull/18))
- *(cli)* accept arxiv as an eprint alias ([#17](https://github.com/binado/bibx/pull/17))
- add the remove command
- add the add command with an INSPIRE provider
- add bibx-core foundations and the list command

### Fixed

- *(add)* do not overwrite concurrent edits to the bib file ([#54](https://github.com/binado/bibx/pull/54))
- *(cli)* warn when malformed entries are skipped ([#56](https://github.com/binado/bibx/pull/56))
- *(core)* reject bibliography symlink writes ([#13](https://github.com/binado/bibx/pull/13))
- return exit code 4 for ambiguous INSPIRE searches ([#9](https://github.com/binado/bibx/pull/9))

### Other

- prepare crates.io packaging ([#55](https://github.com/binado/bibx/pull/55))
- *(error)* keep error sources and separate internal failures ([#26](https://github.com/binado/bibx/pull/26))
- *(list)* parse list columns once ([#25](https://github.com/binado/bibx/pull/25))
- *(discover)* [**breaking**] resolve a single bib file ([#24](https://github.com/binado/bibx/pull/24))
- *(identifier)* make Identifier fields private ([#23](https://github.com/binado/bibx/pull/23))
- *(provider)* make Fetched a validated entry with its key range ([#22](https://github.com/binado/bibx/pull/22))
- *(list)* derive Sortable ordering and simplify Value::new ([#21](https://github.com/binado/bibx/pull/21))
- scaffold the bibx workspace

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/binado/bibx/releases/tag/bibx-core-v0.1.0) - 2026-11-26

### Added

- *(list)* decode LaTeX in table output and sorting ([#20](https://github.com/binado/bibx/pull/20))
- *(cli)* accept arXiv and INSPIRE URLs in bibx add ([#18](https://github.com/binado/bibx/pull/18))
- *(cli)* accept arxiv as an eprint alias ([#17](https://github.com/binado/bibx/pull/17))
- add the remove command
- add the add command with an INSPIRE provider
- add bibx-core foundations and the list command

### Fixed

- *(core)* reject bibliography symlink writes ([#13](https://github.com/binado/bibx/pull/13))
- return exit code 4 for ambiguous INSPIRE searches ([#9](https://github.com/binado/bibx/pull/9))

### Other

- *(error)* keep error sources and separate internal failures ([#26](https://github.com/binado/bibx/pull/26))
- *(list)* parse list columns once ([#25](https://github.com/binado/bibx/pull/25))
- *(discover)* [**breaking**] resolve a single bib file ([#24](https://github.com/binado/bibx/pull/24))
- *(identifier)* make Identifier fields private ([#23](https://github.com/binado/bibx/pull/23))
- *(provider)* make Fetched a validated entry with its key range ([#22](https://github.com/binado/bibx/pull/22))
- *(list)* derive Sortable ordering and simplify Value::new ([#21](https://github.com/binado/bibx/pull/21))
- scaffold the bibx workspace

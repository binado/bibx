# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/binado/bibx/releases/tag/bibx-v0.1.0) - 2026-11-26

### Added

- *(cli)* pull bibliographies from arXiv source ([#38](https://github.com/binado/bibx/pull/38))
- *(cli)* add bibx check and prek hook ([#37](https://github.com/binado/bibx/pull/37))
- *(cli)* show a bibliography entry verbatim ([#36](https://github.com/binado/bibx/pull/36))
- *(list)* warn when no entry has a requested field ([#30](https://github.com/binado/bibx/pull/30))
- *(cli)* print added BibTeX to stdout ([#29](https://github.com/binado/bibx/pull/29))
- *(list)* decode LaTeX in table output and sorting ([#20](https://github.com/binado/bibx/pull/20))
- *(cli)* accept arXiv and INSPIRE URLs in bibx add ([#18](https://github.com/binado/bibx/pull/18))
- *(cli)* accept arxiv as an eprint alias ([#17](https://github.com/binado/bibx/pull/17))
- add the remove command
- add the add command with an INSPIRE provider
- add bibx-core foundations and the list command

### Fixed

- *(cli)* [**breaking**] make the file option subcommand-only ([#16](https://github.com/binado/bibx/pull/16))
- *(cli)* report list stdout errors ([#11](https://github.com/binado/bibx/pull/11))
- return exit code 4 for ambiguous INSPIRE searches ([#9](https://github.com/binado/bibx/pull/9))

### Other

- *(error)* keep error sources and separate internal failures ([#26](https://github.com/binado/bibx/pull/26))
- *(list)* parse list columns once ([#25](https://github.com/binado/bibx/pull/25))
- *(discover)* [**breaking**] resolve a single bib file ([#24](https://github.com/binado/bibx/pull/24))
- *(identifier)* make Identifier fields private ([#23](https://github.com/binado/bibx/pull/23))
- migrate development tasks to mise ([#15](https://github.com/binado/bibx/pull/15))
- scaffold the bibx workspace

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/binado/bibx/releases/tag/bibx-parser-v0.1.0) - 2026-11-27

### Added

- *(core)* gate the INSPIRE URL override behind the testing feature ([#60](https://github.com/binado/bibx/pull/60))
- *(parser)* add a checked Span accessor ([#27](https://github.com/binado/bibx/pull/27))
- *(list)* decode LaTeX in table output and sorting ([#20](https://github.com/binado/bibx/pull/20))
- *(parser)* add a lossless BibTeX parser with spans and error recovery

### Fixed

- *(parser)* skip top-level percent comments ([#10](https://github.com/binado/bibx/pull/10))

### Other

- prepare crates.io packaging ([#55](https://github.com/binado/bibx/pull/55))
- scaffold the bibx workspace

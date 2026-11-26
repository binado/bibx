# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/binado/bibx/releases/tag/bibx-v0.1.0) - 2026-11-26

### Added

- add the remove command
- add the add command with an INSPIRE provider
- add bibx-core foundations and the list command

### Fixed

- *(cli)* [**breaking**] make the file option subcommand-only ([#16](https://github.com/binado/bibx/pull/16))
- *(cli)* report list stdout errors ([#11](https://github.com/binado/bibx/pull/11))
- return exit code 4 for ambiguous INSPIRE searches ([#9](https://github.com/binado/bibx/pull/9))

### Other

- migrate development tasks to mise ([#15](https://github.com/binado/bibx/pull/15))
- scaffold the bibx workspace

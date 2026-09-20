# Changelog

All notable changes to Form 10 Converter are documented in this file.

The project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Ensure packaged desktop builds use Tauri's bundled custom protocol instead of the local development server.

### Added

- Run desktop verification on every push and pull request.
- Publish Windows, macOS, and Linux installers to a GitHub release when a `v*` tag is pushed.

## [0.1.0] - 2026-09-19

### Added

- Local desktop workflow for importing milk-procurement workbooks and exporting FORM-10 `.xlsx` workbooks.
- Native file dialogs, preview generation, validation, and workbook export.
- Windows offline WebView2 installer support.

# Form 10 Converter

A local desktop converter that imports a month-wise milk-procurement workbook and exports a FORM-10 `.xlsx` workbook.

## Workspace

- `crates/form10-core` — UI-independent domain models, source parsing, validation, preview calculations, and FORM-10 generation behind `ConverterService`.
- `crates/form10-ui` — Iced desktop application, file dialogs, rendering, and operating-system file actions. It depends on `form10-core`; the core never depends on the UI.

The workspace produces the `form10-converter` desktop executable.

## Supported files

- Input: `.xls` and `.xlsx`
- Output: `.xlsx`

All processing is local. No workbook data is sent over a network.

## Development

```bash
cargo run -p form10-ui --bin form10-converter
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

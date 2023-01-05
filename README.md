# Rust CLI Quick Start Project

[![coverage](https://shields.io/endpoint?url=https://raw.githubusercontent.com/yourname/rust_cli_quickstart/main/coverage.json)](https://github.com/yourname/rust_cli_quickstart/blob/main/coverage.json)
[![Crates.io](https://img.shields.io/crates/v/rust_cli_quickstart.svg)](https://crates.io/crates/rust_cli_quickstart)
[![Docs.rs](https://docs.rs/rust_cli_quickstart/badge.svg)](https://docs.rs/rust_cli_quickstart)

A template project to quickly start building a Rust CLI tool.

- [x] Based off of `cargo new`
- [x] Uses [`clap`]([) for comma](https://crates.io/crates/clap)nd line processing
- [x] Uses [`colored`](https://crates.io/crates/colored) for messages
- [x] Includes [`lazy_static`](https://crates.io/crates/lazy_static) just because
- [x] Adds a [`Justfile`](https://crates.io/crates/just) with shortcuts
- [x] Includes a [`version.json5`](./version.json5) file for use with [StampVer](https://crates.io/crates/stampver)
- [x] Includes UNLICENSE file
- [x] Adds [`coverage.json`](./coverage.json) and badge
- [x] README with [crates.io](https://crates.io/) and [docs.rs](https://docs.rs/) badges
- [x] Includes a `scratch` directory
- [x] Includes an [`.vscode/launch.json`](.vscode/launch.json) and other settings
- [x] Includes [`.gitignore`](./.gitignore)
- [x] Creates a named binary file
- [x] Default [`log_macros.rs`](./src/log_macros.rs) for logging
- [x] [`rust-toolchain.toml`](./rust-toolchain.toml) set to `stable`
- [x] Uses a [Deno](https://deno.land/) based [`customize.ts`](./customize.ts) file
- [x] Creates a `.vscode/launch.json` file for debugging
- [x] Includes basic unit tests
- [x] Includes [Criterion](https://crates.io/crates/criterion) and basic benchmark tests

# ion

A C/C++ build system (future package manager) written in Rust.

This repository currently implements a CLI with commands like `init`, `build`, `run`, `check`, and `clean`.

## Install

### Download a binary (recommended)

- GitHub Releases: see the **latest release** on your repo’s Releases page.

### Build from source

Prereqs: Rust stable (https://rustup.rs)

```bash
cargo build --release
```

The executable will be at:

- Linux/macOS: `target/release/ion`
- Windows: `target\\release\\ion.exe`

## Usage

```bash
ion --help
```

Project docs site (GitHub Pages): see `docs/`.

## Status

Early / experimental. Expect breaking changes.

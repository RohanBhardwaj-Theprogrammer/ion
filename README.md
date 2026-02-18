
# ion

> Fast, project-local C/C++ build system — written in Rust

---

**ion** is a CLI tool to initialize, build, run, check, and clean C/C++ projects. It uses a simple `.cbuild` config and works cross-platform (Windows & Linux). Early-stage, but already useful for small-to-medium C/C++ codebases.

## Features

- 📦 Project-local config in `.cbuild/`
- 🏗️  One-command project setup: `ion init .`
- ⚡ Fast builds with profiles: `--release`, `--debug`, `--fast`
- 🏃 Run and check targets easily
- 🧹 Clean outputs and extension-matched files safely
- 🖥️  Windows & Linux binaries (see [Downloads](#downloads))

## Quick Start

```sh
# Download a release binary (see below), or build from source:
cargo build --release

# Create a new project
ion init .

# Build and run
ion build .
ion run . -- --help
```

## Commands

- `init`   — Scaffold a new project with default structure
- `build`  — Compile a target (supports profiles)
- `run`    — Build and execute a binary
- `check`  — Syntax-check only (no output binary)
- `env`    — Manage environment variables for build/run
- `clean`  — Remove build outputs and (optionally) extension-matched files

See full docs and command reference at the [docs site](./docs/index.html).

## Downloads

- [Latest Windows/Linux binaries](https://github.com/YOUR-OWNER/YOUR-REPO/releases/latest)
- Or build from source (requires [Rust](https://rustup.rs)):

	```sh
	cargo build --release
	# Windows: target\release\ion.exe
	# Linux:   target/release/ion
	```

## Project Layout

```
myproject/
├── .cbuild/         # Project config
├── src/             # Source files
├── include/         # Headers
├── build/           # Build outputs
```

## Status

**Early/experimental.** Expect breaking changes. Feedback and issues welcome!


# ion

A modern, zero-config build system for C/C++.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> *ion* brings the simplicity of `cargo` and `go build` to C/C++ development.

## Features

- 🚀 **Zero-config builds** — Auto-discovers dependencies from `#include` directives
- ⚡ **Incremental builds** — Only rebuilds what changed
- 👀 **Watch mode** — Auto-rebuild on file save
- 🎯 **Simple CLI** — `ion build`, `ion run`, `ion check`
- 🔧 **Sensible defaults** — Works out of the box, customizable when needed

## Requirements

- Rust (for building ion)
- A C/C++ compiler (`g++`, `clang++`, `gcc`, or `clang`)

---
>Intended Features and workflow

## Quick Start

```bash
# Create a new C++ project
ion init -cpp my_project
cd my_project

# Build and run
ion run .

# Watch for changes (auto-rebuild)
ion notice run .
```

## Commands

| Command | Description |
|---------|-------------|
| `ion init` | Create a new project |
| `ion build` | Compile source files |
| `ion run` | Build and execute |
| `ion check` | Syntax check (no linking) |
| `ion clean` | Remove build artifacts |
| `ion notice` | Watch mode — rebuild on changes |
| `ion log` | View build history |
| `ion env` | Manage build environment |
| `ion config` | Edit project configuration |

## Usage Examples

### Building

```bash
ion build main.cpp                # Build specific file
ion build .                       # Build project default
ion build main.cpp --release      # Release build (optimized)
ion build main.cpp -std 20        # Use C++20
```

### Running

```bash
ion run main.cpp                  # Build and run
ion run main.cpp -- arg1 arg2     # Pass arguments to executable
```

### Watch Mode

```bash
ion notice build main.cpp         # Rebuild on file changes
ion notice run main.cpp           # Rebuild and run on changes
```

## Project Structure

```
my_project/
├── .ion/                # ion configuration (auto-generated)
│   └── config.toml      # Project settings
├── src/                 # Source files
│   └── main.cpp
├── include/             # Header files
└── build/               # Build output (auto-generated)
```

## Configuration

Project settings in `.ion/config.toml`:

```toml
[project]
name = "my_project"
language = "cpp"
standard = 17

[build]
compiler = "g++"
optimization = 0
output_dir = "build"

[build.release]
optimization = 3
flags = ["-DNDEBUG"]

[paths]
sources = ["src"]
includes = ["include"]
```

## Documentation

For detailed design and all features, see the [RFC documentation](rfcs/rfcs0.1.md).

## Roadmap

- [x] Core build system
- [x] Incremental builds
- [x] Watch mode
- [ ] Package management (local dependencies)
- [ ] Git dependencies
- [ ] Central registry integration

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

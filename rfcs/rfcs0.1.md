# RFC 0001: ion — A Modern Build System for C/C++

| | |
| :--- | :--- |
| **RFC Number** | 0001 |
| **Title** | ion Build System Vision |
| **Author** | Rohan |
| **Status** | **Draft** |
| **Created** | 2025-12-05 |

## 1. Summary

**ion**<sup>[1]</sup> is a proposed build system for C and C++ that aims to provide a unified, developer-friendly experience. Its goal is to combine simple project management, intelligent builds with automatic dependency resolution, and a future package manager into a single tool, inspired by the workflow of Rust's `cargo`.

## 2. Motivation

The C/C++ ecosystem is powerful but fragmented. Key pain points this project seeks to address are:
*   **Complex Configuration:** The steep learning curve of build systems like Make and CMake.
*   **Manual Processes:** Easy-to-miss errors from manually tracking `#include` dependencies.
*   **Missing Standard Tooling:** No unified solution for package management, leading to a split between vcpkg, Conan, and others.
*   **Slow Feedback:** The edit-compile-run loop is often manual, slowing down iteration.

## 3. Design Overview

### 3.1 Core Philosophy
The guiding principle is "convention over configuration." **ion** provides strong, sensible defaults that work immediately for standard projects, while remaining customizable for advanced use cases.

### 3.2 Core Commands
The interface is designed around a unified CLI for the primary development loop:

| Command | Purpose |
| :--- | :--- |
| `ion init` | Scaffold a new project with a standard layout and custom layout too throgh json file . |
| `ion build` | Compile, using automatic dependency resolution. |
| `ion run` | Build and then execute the program. |
| `ion check` | Perform a fast syntax check (no linking). |
| `ion notice` | Watch source files and auto-rebuild on changes. |

### 3.3 Key Technical Components
*   **Automatic Dependency Graph:** The core innovation. Parses `#include` directives to build a dependency graph, determining the correct build order and enabling incremental compilation.
*   **Configuration & State:** Project settings and build caches are stored in a `.ion/` directory, using a `config.toml` file.

## 4. Open Questions & Request for Feedback

The primary purpose of this RFC is to gather community input on key design decisions. Your feedback is most valuable on the following points:

1.  **Naming Conflict:** The name "ion" is used by an existing project. **Finding a new, unique, and memorable name is the top priority.** Suggestions are welcome.
2.  **Package Manager Integration:** Should the initial version focus solely on the build system, or is a basic package manager (for local/Git dependencies) essential for an MVP?
3.  **Dependency Resolution Edge Cases:** The automatic `#include` parser is central. What complex, real-world scenarios should it handle from the start? (e.g., generated headers, conditional includes via macros).
4.  **Compiler Interface Strategy:** Should the tool invoke compilers directly, or would generating a `compile_commands.json` file for IDE/tooling integration be more valuable?
5.  **Configuration Format:** Is TOML the right choice for the `config.toml` file, or should we consider YAML or another format?

## 5. Future Possibilities

Beyond the MVP, potential future directions include:
*   A central package registry or integration with existing ones (Conan/vcpkg).
*   Advanced caching for CI/CD pipelines.
*   First-class support for cross-compilation.

---
**Footnotes:**
[1] "ion" is a temporary project name. A rename is required before any public release.

**Next Steps:**
Community feedback on the **Open Questions** will directly shape the project's roadmap. Please share your thoughts on the points above.
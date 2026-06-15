# Specification: Convert Repository from Go to Rust

## Overview
This track specifies the complete rewrite of the `cook-docs` CLI tool from Go (v1.18) to Rust, removing all Go references and codebase artifacts, and replacing them with a Rust-native implementation using `clap` (CLI), `figment`/`config` (configuration), `cooklang` (parsing), and `gtmpl-ng` (Go template compatibility). Development will be carried out on a dedicated Git branch `migrate-to-rust`.

## Branching & Environment
- **Development Branch**: A new branch named `migrate-to-rust` must be created and used for all development work.
- **Runtimes & Toolchain**: Migrate from Go 1.18 compiler to Rust stable (cargo).

## Functional Requirements
1. **CLI Flag & Config Parity**:
   - Re-implement all CLI flags and config options using `clap` and `figment`/`config`:
     - `-d, --dry-run`
     - `-j, --jsonify`
     - `-i, --ignore-file`
     - `-c, --recipe-search-root`
     - `-l, --log-level`
     - `-t, --template-files`
     - `-w, --word-wrap`
     - `-o, --output-dir`
2. **Parsing & Rendering Parity**:
   - Use `cooklang` crate for recipe parsing.
   - Use `gtmpl-ng` for Go-compatible text template rendering, ensuring existing `recipe.md.gotmpl` templates continue to render correctly.
   - Re-implement the directory crawling and ignoring logic (respecting ignore files) in Rust.
3. **Multi-Architecture Support**:
   - The compiled Rust binary must support the same architectures as the Go implementation.

## Non-Functional Requirements
- **Test Parity**: Rewrite unit tests for path resolution, recipe parsing, template rendering, and config parsing in Rust to match the coverage and assertions of the Go test suite.

## Out of Scope
- Introducing new features beyond the Go implementation's behavior (to be implemented later).
- Restructuring the documentation pages themselves.

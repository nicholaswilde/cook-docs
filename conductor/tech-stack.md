# Technology Stack - cook-docs

## Languages & Core Runtimes
- **Rust (Stable)**: The project is written in Rust, using Cargo for dependency management, compilation, and testing.

## Libraries & Frameworks
- **CLI Framework**: [clap](https://github.com/clap-rs/clap) (v4) is used to parse command-line arguments and define flags/subcommands.
- **Configuration**: [figment](https://github.com/SergioBenitez/Figment) is used to load configuration from environment variables, files, and CLI overrides.
- **Cooklang Parsing**: [cooklang-rs](https://github.com/cooklang/cooklang-rs) is used to parse Cooklang recipes.
- **Templating**: [gtmpl-ng](https://github.com/fede1024/gtmpl-rust) is used for rendering recipes using Go template syntax for backwards compatibility.
- **Logging**: [env_logger](https://github.com/rust-cli/env_logger) and `log` crate are used for structured logging.

## Testing & Tooling
- **Testing**: Rust native testing framework (`cargo test`).
- **Documentation**: [Zensical](https://zensical.org/) is used to generate static site documentation for the project, managed via [uv](https://github.com/astral-sh/uv).
- **Task Management**: [Taskfile](https://taskfile.dev/) (`Taskfile.yaml`) is used for automating development tasks (e.g. building, linting, formatting).

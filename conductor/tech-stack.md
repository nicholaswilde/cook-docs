# Technology Stack - cook-docs

## Languages & Core Runtimes
- **Go 1.18+**: The project is written in Go, utilizing standard toolchains for compilation, testing, and dependency management.

## Libraries & Frameworks
- **CLI Framework**: [Cobra](https://github.com/spf13/cobra) is used to structure the CLI command, flags, and help menus.
- **Configuration**: [Viper](https://github.com/spf13/viper) is used to load configuration options from environmental variables, CLI arguments, and YAML files (`.cookdocs`).
- **Cooklang Parsing**: [cooklang-go](https://github.com/aquilax/cooklang-go) parses the Cooklang recipes into structured Go representations.
- **Templating**: Go standard library `text/template` (and `html/template` implicitly) along with [Sprig v3](https://github.com/Masterminds/sprig) to provide extra template helper functions.
- **Logging**: [Logrus](https://github.com/sirupsen/logrus) is used for structured logging across multiple log levels (debug, info, warn, error).

## Testing & Tooling
- **Testing**: [Testify](https://github.com/stretchr/testify) for assertions and mockings in unit tests.
- **Documentation**: [Zensical](https://zensical.org/) is used to generate static site documentation for the project, managed via [uv](https://github.com/astral-sh/uv).
- **Task Management**: [Taskfile](https://taskfile.dev/) (`Taskfile.yaml`) is used for automating development tasks (e.g. building, linting, formatting).

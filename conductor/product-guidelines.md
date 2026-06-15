# Product Guidelines - cook-docs

## User Experience (UX) & CLI Design Principles
- **Actionable CLI Output**: Command output must be clear and helpful. Errors should clearly explain *what* went wrong and *how* to fix it.
- **Dry-run Integrity**: The `--dry-run` flag must guarantee that no files are written or modified on disk. All output must print cleanly to `stdout`.
- **Config Precedence**: Configuration flags must follow a safe hierarchy: CLI flags override environment variables, which override YAML configuration (`.cookdocs`), which override defaults.
- **Log Levels**: Keep logging levels standard:
  - `DEBUG`: Internal execution steps, trace information, and configuration parsing.
  - `INFO`: Standard feedback like "Found recipes..." and progress messages.
  - `WARN`: Recoverable errors (e.g., failed parsing of a single recipe file while others succeed).
  - `ERROR`: Unrecoverable errors (e.g., unable to bind flags, invalid directory path).

## Coding & Architectural Conventions
- **Standard Go Practices**: Follow standard Go guidelines. Code must be formatted with `gofmt` and linted (e.g., using `golangci-lint` or standard pre-commit hooks).
- **Separation of Concerns**: Keep CLI parsing (`cmd/`) separate from the business logic of parsing Cooklang and rendering templates (`pkg/`).
- **Template Safety**: Ensure template rendering is robust against missing metadata or ingredient properties. Default values or safe fallbacks should be handled gracefully rather than causing hard crashes.

## Documentation & Examples
- **Documentation Site**: All user-facing documentation should be built and previewed using MkDocs (configured via `mkdocs.yml`).
- **Standardized Examples**: Provide well-maintained template and recipe examples under `recipe-examples/` so new users can quickly understand how to customize their setup.

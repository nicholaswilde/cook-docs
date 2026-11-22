# cook-docs Project Rules

- **Test-Driven Development (TDD)**: Strictly mandatory. Never write production code without prior failing test:
  1. **Red**: Write failing unit (`#[cfg(test)]` in `src/*.rs`) or integration (`tests/integration_tests.rs`) test; run `rtk test cargo test` and confirm expected failure.
  2. **Green**: Write minimum code passing test (Ponytail discipline).
  3. **Refactor**: Clean up without changing behavior; re-verify `task fmt`, `task lint`, `task test`.
- **Stack Constraints**: Retain `clap`, `figment`, `cooklang`, `gtmpl-ng`; preserve Go template compatibility and `.cookdocsignore` / `--output-dir` behavior.
- **Cross-Test Guard**: Guard any child binary invocation in integration tests so `cross` targets skip gracefully when binfmt/sysroot absent.
- **Docs Sync (`docs/`)**: When adding, updating, or removing CLI flags, config keys, template fields/helpers, or behavior, audit and update all affected `docs/` pages (`usage.md`, `configuration.md`, `templates.md`, `README.md`, `index.md`, `about.md`) plus `assets/config.example.toml`, `completions/*`, and `assets/cook-docs.1` when CLI/config changes; verify `task lint:md` (`rumdl check .`) and `uv run zensical build --clean`.

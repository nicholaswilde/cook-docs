# cook-docs Project Rules

- **Test-Driven Development (TDD)**: Strictly mandatory. Never write production code without prior failing test:
  1. **Red**: Write failing unit (`#[cfg(test)]` in `src/*.rs`) or integration (`tests/integration_tests.rs`) test; run `rtk test cargo test` and confirm expected failure.
  2. **Green**: Write minimum code passing test (Ponytail discipline).
  3. **Refactor**: Clean up without changing behavior; re-verify `task fmt`, `task lint`, `task test`.
- **Stack Constraints**: Retain `clap`, `figment`, `cooklang`, `gtmpl-ng`; preserve Go template compatibility and `.cookdocsignore` / `--output-dir` behavior.
- **Cross-Test Guard**: Guard any child binary invocation in integration tests so `cross` targets skip gracefully when binfmt/sysroot absent.

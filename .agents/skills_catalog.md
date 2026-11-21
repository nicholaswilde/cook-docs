# Skills Catalog

| Skill | Description | Task / Command |
|---|---|---|
| [`lint-fmt-fix`](skills/lint_fmt_fix/SKILL.md) | Run formatting (`task fmt`), clippy auto-fixes, linting (`task lint`), and tests (`task test`). | `task fmt && task lint && task test` |
| [`release`](skills/release/SKILL.md) | Automates versioning, tagging, and deployment via `Cargo.toml` bump and atomic push. | `/release` |
| [`release-summary`](skills/release-summary/SKILL.md) | Generates categorized release notes via `scripts/release_summary.py` and updates the GitHub release via `gh`. | `/release-summary` / `task release:summary` |

# Specification: Convert static website from mkdocs-material to zensical

## Overview
This track involves migrating the project's static documentation site from `mkdocs-material` (running via Docker) to `Zensical` (running locally via Python and managed with `uv`). The `mkdocs.yml` configuration will be translated to a new `zensical.toml` in the repository root, and `Taskfile.yaml` will be updated to replace MkDocs commands with Zensical commands.

## Functional Requirements
1. **Python Environment Setup**:
   - Initialize a python project using `uv` in the root directory, creating `pyproject.toml` and `uv.lock`.
   - Add `zensical` as a dependency in `pyproject.toml`.
2. **Configuration Migration**:
   - Translate all site metadata (`site_name`, `site_description`, `site_author`, `site_url`, `copyright`, `repo_url`, `repo_name`) from `mkdocs.yml` to `zensical.toml` under the `[project]` section.
   - Define navigation in `zensical.toml` matching the exact page order from `mkdocs.yml`.
   - Configure equivalent markdown extensions (toc, tables, admonition, math/arithmatex, etc.) in `zensical.toml`.
3. **MkDocs Cleanup**:
   - Delete the obsolete `mkdocs.yml` file from the repository root.
4. **Development Workflow Integration**:
   - Update `Taskfile.yaml` `serve` task to run `uv run zensical serve` instead of running `mkdocs-material` via Docker.

## Acceptance Criteria
- Running `uv run zensical build` succeeds and builds the site in the output directory.
- Running `task serve` runs the Zensical development server successfully on the configured port.
- Site metadata and navigation match the original MkDocs structure exactly.
- Markdown features (tables, admonitions, and math equations) are parsed correctly.

## Out of Scope
- Rewriting or restructuring the markdown files under `docs/`.
- Deploying the site to production.

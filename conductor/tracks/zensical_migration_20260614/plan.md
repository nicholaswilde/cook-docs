# Implementation Plan: Convert Static Website from mkdocs-material to zensical

## Phase 1: Python Environment & Dependencies Setup
- [ ] Task: Initialize python environment and add zensical
    - [ ] Run `uv init` in the repository root to create `pyproject.toml`
    - [ ] Run `uv add zensical` to add zensical as a dependency and generate `uv.lock`
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Python Environment & Dependencies Setup' (Protocol in workflow.md)

## Phase 2: Configuration Migration
- [ ] Task: Create zensical.toml and migrate configurations
    - [ ] Translate metadata, navigation, and extra files from `mkdocs.yml` to `zensical.toml`
    - [ ] Configure markdown extensions and extra scripts in `zensical.toml`
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Configuration Migration' (Protocol in workflow.md)

## Phase 3: Integration & Cleanup
- [ ] Task: Update Taskfile.yaml and remove mkdocs.yml
    - [ ] Update the `serve` task in `Taskfile.yaml` to run zensical instead of mkdocs via Docker
    - [ ] Delete `mkdocs.yml`
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Integration & Cleanup' (Protocol in workflow.md)

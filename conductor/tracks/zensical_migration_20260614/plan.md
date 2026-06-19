# Implementation Plan: Convert Static Website from mkdocs-material to zensical

## Phase 1: Python Environment & Dependencies Setup [checkpoint: 75d8f98]
- [x] Task: Initialize python environment and add zensical [aa22f46]
    - [x] Run `uv init` in the repository root to create `pyproject.toml`
    - [x] Run `uv add zensical` to add zensical as a dependency and generate `uv.lock`
- [x] Task: Conductor - User Manual Verification 'Phase 1: Python Environment & Dependencies Setup' (Protocol in workflow.md) [75d8f98]

## Phase 2: Configuration Migration [checkpoint: 7d269b5]
- [x] Task: Create zensical.toml and migrate configurations [41313d2]
    - [x] Translate metadata, navigation, and extra files from `mkdocs.yml` to `zensical.toml`
    - [x] Configure markdown extensions and extra scripts in `zensical.toml`
- [x] Task: Conductor - User Manual Verification 'Phase 2: Configuration Migration' (Protocol in workflow.md) [7d269b5]

## Phase 3: Integration & Cleanup [checkpoint: 583b7a3]
- [x] Task: Update Taskfile.yaml and remove mkdocs.yml [73cc03f]
    - [x] Update the `serve` task in `Taskfile.yaml` to run zensical instead of mkdocs via Docker
    - [x] Delete `mkdocs.yml`
- [x] Task: Conductor - User Manual Verification 'Phase 3: Integration & Cleanup' (Protocol in workflow.md) [583b7a3]

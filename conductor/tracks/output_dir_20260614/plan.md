# Implementation Plan: Implement Output Directory Configuration

## Phase 1: Setup & Configuration [checkpoint: 3e6f850]
- [x] Task: Update configuration definitions and CLI flags [050d30c]
    - [x] Write tests verifying that configuration parsing correctly captures `--output-dir` flag and viper bindings.
    - [x] Update `types.Config` struct to include `OutputDir` field and register `--output-dir` / `-o` flags.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Setup & Configuration' (Protocol in workflow.md) [3e6f850]

## Phase 2: Output Path Resolution Logic
- [ ] Task: Update output path resolution to handle custom output directory
    - [ ] Write unit tests verifying output path resolution with and without `OutputDir` setting.
    - [ ] Modify path generation in `pkg/cook` to compute the relative path of the recipe from the search root and combine it with the configured `OutputDir`.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Output Path Resolution Logic' (Protocol in workflow.md)

## Phase 3: Directory Creation & Integration
- [ ] Task: Ensure parent directories are created before file write
    - [ ] Write integration test verifying that when generating document to a non-existent subdirectory inside `OutputDir`, directories are created successfully.
    - [ ] Modify `pkg/document/generate.go` to create parent directories of the target output file before writing.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Directory Creation & Integration' (Protocol in workflow.md)

---
name: lint-fmt-fix
description: Run formatting (task fmt) and linting (task lint), and automatically apply fixes (including cargo clippy --fix) for formatting and lint issues.
---

## Overview
This skill automates formatting and lint checks for the project, and automatically applies fixes for formatting and compiler warnings/errors where possible.

## Execution Steps

1. **Format Code**:
   Run the project's formatting command:
   ```bash
   task fmt
   ```

2. **Auto-Fix Clippy Issues**:
   Run Cargo Clippy with automatic fixes enabled:
   ```bash
   cargo clippy --fix --allow-dirty --allow-staged --all-targets
   ```

3. **Verify Linting**:
   Run the project's linting command to verify all checks pass:
   ```bash
   task lint
   ```

4. **Verify Test Suite**:
   Run the tests to ensure the changes did not break anything:
   ```bash
   task test
   ```

5. **Report & Commit**:
   - If any files were modified by the formatting or auto-fix steps, list the changed files and stage/commit them with a clear message like `style: auto-format and fix clippy warnings`.
   - If any manual fixes are required (i.e. Clippy could not fix them automatically), report them to the user or fix them manually.

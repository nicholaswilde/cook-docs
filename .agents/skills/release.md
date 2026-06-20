# /release

Automates the versioning, tagging, and deployment process for the project.

## Description
This skill handles the end-to-end release process: extracting the current version, incrementing the patch version, updating configuration, committing changes, creating git tags, and pushing atomically to the remote repository.

## Protocol

1. **Extract and Calculate Version:**
   - Read `Cargo.toml` to locate the `version` field (e.g., `version = "0.9.1"`).
   - Calculate the new patch version by incrementing the last number (e.g., `0.9.1` -> `0.9.2`).

2. **Update Files:**
   - Update the `version` field in `Cargo.toml` with the new version.
   - Run `cargo check` to automatically update the version in `Cargo.lock`.

3. **Pre-release Validation:**
   - Run `task fmt` to format the files.
   - Run `task lint` to ensure clippy is clean.
   - Run `task spellcheck` to ensure no typos exist.
   - Run `task test` to guarantee all tests pass before making the release.

4. **Verify Git State:**
   - Check `git status --porcelain` to ensure there are no unexpected local modifications.
   - Run `git pull --rebase` to ensure the local branch is synchronized with `origin main`.

5. **Commit and Tag Changes:**
   - Stage `Cargo.toml` and `Cargo.lock`.
   - Commit the changes with the exact message: `chore: Bump version to <new_version>`.
   - Create an annotated git tag: `git tag -a v<new_version> -m "<new_version>"`.
   - **NOTE:** Use non-interactive command flags (e.g., `git commit -m`, `git tag -a -m`) to prevent terminal prompts or editor spawning.

6. **Atomic Push:**
   - Push the branch and the new tag atomically:
     `git push --atomic origin main v<new_version>`

7. **Error Handling:**
   - If any step fails, stop immediately, do not push, and report the detailed error to the user.

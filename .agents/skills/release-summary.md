# /release-summary [range]

Generates a clean, professional GitHub release summary based on git logs.

## Description
This skill retrieves the git commit logs for a specified range (or defaults to the latest two tags) and formats them into a categorized, structured release summary suitable for GitHub.

## Protocol

1. **Determine the Git Log Range:**
   - Check if a `[range]` argument (e.g., `v0.9.0..v0.9.1` or `HEAD`) is provided.
   - If no range is specified, execute git commands to detect the latest two tags:
     - Run `git tag --sort=-v:refname | head -n 2`
     - Extract `LATEST` (the first line) and `PREVIOUS` (the second line).
     - If `PREVIOUS` is empty (only one tag or no tags), default `range` to `HEAD`.
     - Otherwise, set `range` to `PREVIOUS..LATEST`.

2. **Retrieve Git Log Entries:**
   - Run `git log --pretty=format:"- %s" <range>` where `<range>` is the determined range.

3. **Generate and Format the Release Summary:**
   - Categorize the commit messages into the following markdown sections (using `###` for headers):
     - ### 🚀 **New Features** (for commits starting with `feat`)
     - ### 🐛 **Bug Fixes** (for commits starting with `fix`)
     - ### ✨ **Improvements** (for commits starting with `refactor`, `perf`, `style`)
     - ### 📝 **Documentation** (for commits starting with `docs`)
   - Clean up the commit messages:
     - Exclude any commit messages related to `conductor` or `checkpoint` tasks (e.g., commits containing `conductor` or `checkpoint` in their type, scope, or message body).
     - Strip the type prefix (e.g. "feat:", "fix:").
     - If a scope is present (e.g., "auth:"), keep it (e.g., `auth: add token validation` -> `auth: Add token validation`).
     - Ensure the first letter of each bullet is capitalized.
     - Use backticks for technical terms (e.g., `cooklang`, `gtmpl-ng`, `clap`, `figment`, `serde`, `serde_json`, `walkdir`).
   - Format restrictions:
     - Do not include line numbers in the output.
     - Use standard triple backticks (```) for any code blocks.
     - Omit sections that have no matching changes.
     - Do not include git commit hashes.
     - Maintain a direct, professional, and technical tone.

4. **Add Changelog Comparison Link:**
   - At the bottom of the summary, add a comparison link:
     `**Full Changelog**: https://github.com/nicholaswilde/cook-docs/compare/<url_range>`
     where `<url_range>` is the range with `..` replaced by `...` (e.g., `v0.9.0...v0.9.1` or `HEAD`).

---
name: release-summary
description: Generates clean GitHub release summary based on git logs using scripts/release_summary.py and updates the GitHub draft or pre-release via gh. Trigger: /release-summary, "release summary", "generate release notes".
---

# /release-summary [range]

Generates a clean, professional GitHub release summary based on git logs and updates the target GitHub release (draft, pre-release, or published) via `gh`.

## Description

This skill uses `scripts/release_summary.py` to retrieve git commit logs, format them into a structured Markdown summary (New Features, Bug Fixes, Improvements, Documentation, Special Thanks, Full Changelog), and update the existing GitHub release via `gh release edit` (preserving its draft/pre-release status, or creating a draft pre-release if none exists yet).

## Protocol

1. Execute the Python script via `uv run` (or `task`). If no range is provided, it automatically detects the latest two tags.

   ```bash
   rtk uv run python3 scripts/release_summary.py [range]
   # or
   task release:summary -- [range]
   ```

   Supported range formats: `v0.9.2..v0.9.4`, `v0.9.2 to v0.9.4`, `v0.9.4` (diff from previous tag), or omitted (latest two tags).

2. **Preview Without Updating (`--dry-run`)**: Print generated Markdown without calling `gh`:

   ```bash
   rtk uv run python3 scripts/release_summary.py [range] --dry-run
   ```

3. **Special Thanks**: If the user asks to add special thanks (e.g. to a contributor or tester), pass `--thanks` (and optionally `--for` / reason):

   ```bash
   # Name and reason
   rtk uv run python3 scripts/release_summary.py [range] --thanks "<Name>" --for "<reason>"

   # Or full message
   rtk uv run python3 scripts/release_summary.py [range] --thanks "Huge thanks to **<Name>** for <reason>!"
   ```

   This generates the section:

   ```markdown
   ### 🙏 **Special Thanks**

   Huge thanks to **<Name>** for <reason>!
   ```

4. **Update Release State (Optional)**: By default, `gh release edit` updates notes in-place while preserving whether the release is a draft, pre-release, or published release. Pass `--draft` or `--prerelease` to explicitly set those flags on edit. If no GitHub release exists for the target tag yet, the script creates a draft pre-release (`gh release create --draft --prerelease`).

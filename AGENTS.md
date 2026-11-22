# AGENTS.md

Conventions and architecture for AI agents working on `cook-docs`.

---

<!-- rtk-begin -->
## RTK Command Guidelines

- **Git Operations**: Prefix `git` commands with `rtk` (e.g., `rtk git status`, `rtk git diff`, `rtk git log`, `rtk git commit`, `rtk git push`).
- **GitHub CLI**: Prefix `gh` commands with `rtk` and pipe to `cat` (e.g., `rtk gh issue list | cat`, `rtk gh pr status | cat`). Always pipe to bypass interactive pagers.
- **File & Directory Inspection**: Use `rtk ls`, `rtk tree`, `rtk find`, `rtk fd`, or `rtk read` when listing or reading files for token-optimized output.
- **Searching**: Use `rtk rg` or `rg` (ripgrep) for line search pattern matching to save tokens.
- **Spellchecking**: Run `rtk typos --format brief`; `--format brief` strips visual code context blocks to conserve tokens.
- **Link Checking**: Run `rtk lychee --no-progress [optional: --offline] <target>`; always pass `--no-progress` to suppress terminal escape sequences, and `--offline` when validating local file links.
- **TOML Operations**: Prefix `taplo` commands with `rtk` (e.g., `rtk taplo fmt`, `rtk taplo lint`, `rtk taplo check`).
- **Markdown Operations**: Prefix `rumdl` commands with `rtk` (e.g., `rtk rumdl check .`, `rtk rumdl fmt .`).
- **Ripwire Operations**: Prefix `ripwire` commands with `rtk` (e.g., `rtk ripwire <dir> --for="<task>"`). Always invoke via CLI, never register or use MCP.
- **Build & Test Outputs**: Use `rtk err` or `rtk test` when running build/test commands to filter output to errors/failures only (e.g., `rtk test cargo test`, `rtk test task test`).
- **Python & UV Operations**: Prefix Python scripts and commands with `uv run`, and prefix `uv` commands with `rtk` (e.g., `rtk uv run <script.py>`, `rtk uv run zensical serve`, `rtk uv add`, `rtk uv sync`). Never run bare `python` or bare `uv`.
- **Exceptions**: Inside `ctx_execute` / `ctx_batch_execute` sandbox code, skip `rtk` (only printed output enters context). Drop `rtk` when exact raw bytes or patch required (`git diff` patch).
<!-- rtk-end -->

<!-- context-mode-begin -->
## Context-Mode Routing Guidelines

- **Derive, Do Not Dump**: Do NOT use `ctx_execute_file` or `ctx_execute` to dump whole file or full config. Print only specific value, matches, count, or known line range needed.
- **Tool Call Surface**: When using MCP wrappers, call `call_mcp_tool` with `ServerName: "context-mode_context-mode"` (or `"context-mode"`) and `ToolName: "ctx_execute"`, `"ctx_execute_file"`, `"ctx_batch_execute"`, `"ctx_fetch_and_index"`, `"ctx_search"`, or `"ctx_index"`.
- **Mandatory Routing**:
  - For analyze/count/filter/compare/search/parse/transform tasks, write code with `ctx_execute` and print only final answer.
  - For analyzing/exploring inside file, use `ctx_execute_file`. Use native `read_file` only when editing requires exact bytes or small known range.
  - Use `ctx_batch_execute` for multi-command repository reconnaissance.
  - Use `ctx_execute` for shell commands whose output may exceed short fixed answer.
  - Use `ctx_fetch_and_index` for web content, then `ctx_search` to query it.
  - Return only derived answers, concise summaries, selected snippets, or file paths.
<!-- context-mode-end -->

<!-- ponytail-begin -->
## Ponytail (Lazy Senior Dev Mode) Guidelines

- **Stop at first rung that holds**:
  1. Need to build at all? (YAGNI)
  2. Already in codebase? Reuse existing helpers/utils/patterns.
  3. Stdlib does it? Use it.
  4. Native platform feature covers it? Use it.
  5. Installed dependency solves it? Use it.
  6. Can be one line? One line.
  7. Only then: write minimum working code.
- **Bug fix = root cause, not symptom**: Fix shared function/path rather than individual callers.
- **Rules**:
  - No unrequested abstractions, boilerplate, or avoidable dependencies.
  - Deletion over addition. Boring over clever. Fewest files possible.
  - Shortest working diff wins once problem understood.
  - Mark deliberate simplifications cutting real corner with `ponytail:` comment naming ceiling and upgrade path.
  - Ensure logic leaves behind ONE runnable check (assert-based demo/self-check or small test file). Trivial one-liners need none.
<!-- ponytail-end -->

<!-- caveman-begin -->
## Caveman Guidelines

Respond terse like smart caveman. All technical substance stay. Only fluff die.

Rules:

- Drop: articles (a/an/the), filler (just/really/basically), pleasantries, hedging.
- Fragments OK. Short synonyms. Technical terms exact. Code unchanged.
- Pattern: [thing] [action] [reason]. [next step].
- Not: "Sure! I'd be happy to help you with that."
- Yes: "Bug in auth middleware. Fix:"

Switch level: /caveman lite|full|ultra|wenyan
Stop: "stop caveman" or "normal mode"

Auto-Clarity: drop caveman for security warnings, irreversible actions, user confused. Resume after.

Boundaries: code/commits/PRs written normal.
<!-- caveman-end -->

<!-- codegraph-begin -->
## CodeGraph

In repositories indexed by CodeGraph (`.codegraph/` directory exists at repo root), reach for it BEFORE grep/find or reading files when exploring code:

- **MCP tool**: `codegraph_explore` answers source + call path + blast radius in one call. Dynamic-dispatch hops included. Name file or symbol in query to read current line-numbered source.
- **Shell**: `codegraph explore "<symbol names or question>"`.

If no `.codegraph/` directory exists, skip CodeGraph entirely.
<!-- codegraph-end -->

<!-- serena-begin -->
## Serena Semantic Navigation & LSP Integration

Use Serena (`serena`) MCP tools for semantic navigation, AST symbol inspection, LSP diagnostics, and project memories:

1. **Map first**: Call `get_symbols_overview` on target file instead of reading entire file.
2. **Read target symbol only**: Call `find_symbol` with `include_body: true` for target function/class/method.
3. **Check callers**: Call `find_referencing_symbols` or `find_implementations` before signature or behavior changes.
4. **Edit by symbol**: Use `replace_symbol_body`, `insert_after_symbol`, `insert_before_symbol`, or `rename_symbol`. Use standard edit for small tweaks or non-code files.
5. **Verify diagnostics**: Call `get_diagnostics_for_file` after edits to catch LSP errors before full test run.
<!-- serena-end -->

<!-- headroom-begin -->
## Headroom MCP (Context Window Compression)

Use Headroom (`headroom`) MCP tools to manage context window consumption on large payloads:

- `headroom_compress`: Shrink large tool outputs or raw data.
- `headroom_retrieve`: Retrieve original content by hash if specific details needed.
- `headroom_stats`: Monitor headroom cache usage.
<!-- headroom-end -->

<!-- ripwire-begin -->
## Ripwire CLI Guidelines

Use `ripwire` CLI (prefixed with `rtk ripwire`) for AST codebase mapping, call graphs, blast radius, and structural queries:

- **CLI First**: Always invoke `rtk ripwire <dir> [flags]` via CLI; do NOT use MCP server.
- **Task Orientation**: `rtk ripwire <dir> --for="<task>"` for ranked signatures and bodies.
- **Task Bundle**: `rtk ripwire <dir> --pack-task="<task>" --legend=compact`.
- **Stack Trace / Error**: `rtk ripwire <dir> --from-trace=FILE --legend=compact` (`-` for stdin).
- **Call Graph**: `rtk ripwire <dir> --callers=SYM`, `--callees=SYM`, `--uses=SYM`.
- **Blast Radius**: `rtk ripwire <dir> --impact=SYM --legend=compact` before changing shared code.
- **Quality Delta & Gate**: `rtk ripwire <dir> --quality-delta --legend=compact` and `--test-gate --legend=compact`.
- **Atomic Edits**: `rtk ripwire <dir> --replace-symbol-body=SYM --edit-payload=FILE` without whole-file re-reading.
<!-- ripwire-end -->

<!-- ripgrep-guidelines-begin -->
## Ripgrep Guidelines

- **Pattern Searching**: Always use `rg` (ripgrep) or `rtk rg` instead of standard `grep` to search text inside files quickly and save token output.
- **Scope**: Use `-t rust` for Rust files, `-l` for file listing, keep context small (`-C 1`).
<!-- ripgrep-guidelines-end -->

<!-- taplo-rules-begin -->
## Taplo TOML Rules

- **Prefix with RTK**: Always prefix `taplo` commands with `rtk` in interactive shells (e.g., `rtk taplo fmt`, `rtk taplo lint`).
- **Format**: Run `rtk taplo fmt` to format TOML files in-place (`Cargo.toml`, `pyproject.toml`, `zensical.toml`).
- **Validation**: Run `rtk taplo lint` to validate TOML syntax and schema.
<!-- taplo-rules-end -->

<!-- uv-rules-begin -->
## Python & `uv` Rules

- **Prefix with `uv run`**: Always prefix Python scripts and commands with `uv run` (e.g., `rtk uv run zensical serve`, `rtk uv run <script.py>`). Never run bare `python`, `python3`, `pip`, `pipx`, or `poetry`.
- **Sync Lockfile**: Whenever `pyproject.toml` dependencies or configurations change, run `rtk uv lock` (or `rtk uv sync`) so `uv.lock` and `pyproject.toml` stay synchronized.
<!-- uv-rules-end -->

<!-- issue-management-begin -->
## GitHub Issue Management

When creating or updating GitHub issues, adhere to naming and labeling scheme:

- **Features**: Prefix title with `[feat]:` and apply `enhancement` label.
- **Bugs**: Prefix title with `[bug]:` and apply `bug` label.
- **Commit Closing**: GitHub issues only close automatically when commits containing `Fixes #<number>` are pushed to remote repository.
<!-- issue-management-end -->

<!-- sensitive-information-protection-begin -->
## Sensitive Information & Secret Protection

NEVER commit or stage sensitive credentials or secrets:

1. **Secrets & Credentials**: Never commit `.env` files, API keys, seeds, passwords, or private encryption keys.
2. **Explicit Staging**: Never use `git add .` or blanket staging when untracked secret files exist; stage explicitly by name.
<!-- sensitive-information-protection-end -->

---

## 1. Project Overview & Architecture

**`cook-docs`**: Rust CLI generating standardized Markdown documentation from Cooklang (`.cook`) recipes using Go-compatible templates (`gtmpl-ng`).

- `src/main.rs`: CLI entrypoint, logger init, recipe discovery, serial (`--dry-run`) or threaded rendering.
- `src/lib.rs`: Library root (`config`, `cook`, `document`, `types`).
- `src/config.rs`: CLI (`clap`), config file (`.cookdocs`, `.cookdocs.yml`, `.cookdocs.yaml`), and env (`figment`) loading.
- `src/cook.rs`: `.cook` file discovery (`.cookdocsignore` aware), path resolution, Cooklang parsing.
- `src/document.rs`: `gtmpl-ng` template loading, custom template funcs, Markdown formatting and output.
- `src/types.rs`: `Config`, `Recipe`, `Info`, `Ingredient`, `Cookware`, `Timer`, `Step`.
- `docs/` & `zensical.toml`: Static docs site built with Zensical via `uv`.
- `.agents/conductor/`: Archived legacy conductor tracks and workflow docs.

---

## 2. Technology Stack

- **Language & Edition:** Rust (Edition 2024)
- **CLI & Config:** `clap` v4 (`derive`, `env`), `figment` (`toml`, `yaml`, `env`)
- **Parsing & Templates:** `cooklang`, `gtmpl-ng`, `serde`, `serde_json`
- **Discovery & Logging:** `ignore`, `walkdir`, `log`, `env_logger`
- **Docs Site:** Python >=3.12 (`pyproject.toml`, `uv.lock`), `zensical` via `uv`
- **Task Automation:** `task` (`Taskfile.yaml`)

---

## 3. Testing & Quality Gates

### Test-Driven Development (TDD) Protocol

1. **Red Phase:** Write failing unit (`src/*.rs` `#[cfg(test)]`) or integration (`tests/`) test first. Run `rtk test cargo test` to verify failure.
2. **Green Phase:** Write minimum code passing tests (Ponytail discipline).
3. **Refactor Phase:** Clean up while keeping tests green.

### Quality Gates

Before completing any task:

- [ ] `task fmt` / `cargo fmt --all --check` passes.
- [ ] `task lint` (`yamllint-rs .` and `cargo clippy -- -D warnings`) passes.
- [ ] `task test` (`cargo test`) passes.
- [ ] `task spellcheck` (`rtk typos --format brief`) passes.
- [ ] `docs/` synced for added/changed/removed features; `task lint:md` (`rumdl check .`) and `uv run zensical build --clean` pass.

---

## 4. Development Commands (`Taskfile.yaml`)

```bash
task test         # Run Rust tests
task fmt          # Format Rust code
task lint         # Run yamllint-rs and clippy (-D warnings)
task spellcheck   # Run typos spellcheck
task dry-run      # Dry-run cook-docs on testdata
task serve        # Serve Zensical docs via uv run
task dist         # Build release binary
task build-all    # Cross-compile Linux, macOS, Windows targets
```

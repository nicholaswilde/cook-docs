# Usage

!!! warning
    The mode of operation of `cook-docs` is to process all recipes in the
    working directory and sub folders. See [Mode of Operation][3] for
    details.

There are two key parameters to know when running `cook-docs`:

- `--recipe-search-root` (`-c`) specifies the directory under which the tool recursively searches for `.cook` recipe files (default `.`).
- `--template-files` (`-t`) specifies a comma-separated list of Go-compatible template files used to render each recipe's Markdown output (default `recipe.md.gotmpl`).

If a template entry is a bare filename (like `recipe.md.gotmpl`), it is resolved starting in the recipe's directory and walking up parent directories to `recipe-search-root` (inclusive) before falling back to the built-in template. If it is a relative path starting with `.` (like `./_templates.gotmpl`), it is resolved relative to `recipe-search-root`.

## :runner: Generating Documentation

Generate Markdown files for all `.cook` recipes under the current directory:

```bash
cook-docs
```

Preview rendered Markdown on stdout without writing any files:

```bash
cook-docs --dry-run
```

Render recipes from a specific directory into a separate output folder (mirroring relative subdirectories):

```bash
cook-docs --recipe-search-root ./recipes --output-dir ./docs/recipes
```

Render one or more specific recipes while keeping `--recipe-search-root` as the base for mirrored output paths, and optionally copy companion images (`.webp`, `.jpg`, `.jpeg`, `.png`) into an assets directory:

```bash
cook-docs -c ./cook -o ./docs --assets-dir ./docs/assets/images ./cook/breads/Bagels.cook
# Override Info.ImageSlug extension when running external WebP conversion afterwards:
cook-docs -c ./cook -o ./docs --assets-dir ./docs/assets/images --image-ext webp ./cook/breads/Bagels.cook
```

Inspect parsed recipe data as JSON on stdout (single object for one recipe, JSON array for multiple):

```bash
cook-docs --jsonify | jq .
```

Use an explicit configuration file (TOML, YAML, or JSON):

```bash
cook-docs --config .cookdocs.toml
```

## :eyes: Watch Mode

Monitor `recipe-search-root` and configured `template-files` and automatically re-render affected recipes on save:

```bash
cook-docs watch
# OR
cook-docs --watch
```

Customize the debounce delay (default `300` ms) and combine with standard flags:

```bash
cook-docs watch --recipe-search-root ./recipes --output-dir ./docs --debounce 500
```

Editing a single `.cook` file re-renders only that recipe; editing a template file re-renders recipes in that directory and nested subdirectories; editing a shared/external template re-renders all discovered recipes. Press `Ctrl+C` to stop.

## :stethoscope: Doctor Diagnostics

Validate configuration files, `recipe-search-root`, templates, and output directory permissions:

```bash
cook-docs doctor
# Alias:
cook-docs doc
# Machine-readable JSON output:
cook-docs doctor --json
```

Exits non-zero when errors are detected.

## :arrow_up: Self-Update

Check for or install the latest release from GitHub:

```bash
# Check only without installing
cook-docs self-update --check

# Update interactively (or skip prompt with -y/--yes)
cook-docs self-update --yes

# Pin a specific release tag/version
cook-docs self-update --version 0.9.3 --yes
```

Alias: `cook-docs update`.

## :shell: Completions & Man Page

```bash
# Generate shell completions (bash, zsh, fish, powershell, elvish)
cook-docs completions bash

# Generate UNIX roff man page
cook-docs man
```

## :file_folder: Ignoring Recipe Directories

`cook-docs` supports a `.cookdocsignore` file (`.gitignore` syntax) to exclude directories or specific `.cook` files from discovery. Parent directories listed in `.cookdocsignore` skip all recipes underneath them. Use `-i, --ignore-file` to specify an alternate ignore filename.

## :grey_question: Help

Use `--help` on the root command or any subcommand:

```bash
cook-docs --help
cook-docs watch --help
```

## :robot: Task

[go-task][1] automates development workflows (`test`, `fmt`, `lint`, `lint:md`, `coverage`, `completions`, `man`, `serve`, `release`).

Run `task` to list all available tasks:

```bash
task
```

## :material-git: pre-commit

[pre-commit][2] may also be used on this project:

```bash
pre-commit install
pre-commit install-hooks
```

[1]: https://taskfile.dev/#/
[2]: https://pre-commit.com/
[3]: ./about.md#mode-of-operation

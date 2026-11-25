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
# Enrich output with emojis, gram weights, and internal recipe cross-links (ingredients + step directions):
cook-docs -c ./cook -o ./docs --emoji-file ./includes/emoji.yaml --weights-file ./includes/weights.yaml --link-dirs sauces-and-dressings,ingredients --link-in-steps ./cook/mains/Tacos.cook
```

Inspect parsed recipe data as JSON on stdout (single object for one recipe, JSON array for multiple):

```bash
cook-docs --jsonify | jq .
```

Use an explicit configuration file (TOML, YAML, or JSON):

```bash
cook-docs --config .cookdocs.toml
```

## :twisted_rightwards_arrows: Non-Destructive Merge Mode

By default, `cook-docs` overwrites existing Markdown files. Pass `--merge` (or set `merge = true` in `.cookdocs.toml`) to update generated recipe sections without clobbering manual edits:

```bash
# Merge freshly rendered content into existing Markdown files
cook-docs -c ./cook -o ./docs --merge ./cook/breads/Bagels.cook

# Safe live watch mode against manually enriched Markdown docs
cook-docs watch -c ./cook -o ./docs --merge

# Preview a merged result on stdout without modifying disk
cook-docs -c ./cook -o ./docs --merge --dry-run ./cook/breads/Bagels.cook

# Force a full overwrite even when `merge = true` in config
cook-docs -c ./cook -o ./docs --force ./cook/breads/Bagels.cook
```

`--merge` supports two modes automatically:

1. **HTML region markers**: if an existing Markdown file contains `<!-- cook-docs:begin -->` and `<!-- cook-docs:end -->`, only the content between those markers is replaced.
2. **Structure-aware merge**: when no markers are present, `cook-docs` merges YAML frontmatter (unioning `tags` and preserving manual keys such as `hero`), preserves manual H1 emojis and image embeds (dropping orphaned reference definitions such as `[1]:` when a compiled template switches a preamble image from reference-style to inline), preserves custom `##` sections in relative order, keeps manual `=== "..."` content tabs, merges `## Source` bullets without duplicating URLs, and preserves manual admonitions (`!!!`/`???`), `mermaid` diagrams, reference links, and footnotes.

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

Validate configuration files, `recipe-search-root`, templates, emoji/weights files, and output directory permissions:

```bash
cook-docs doctor
# Alias:
cook-docs doc
# Machine-readable JSON output:
cook-docs doctor --json
```

Pass `--check-recipes` (with optional positional `.cook` files or directories) to run recipe-level validation checks, `--only <CATEGORIES>` to filter specific check categories (`syntax`, `images`, `metadata`, `emojis`, `ignored`), and `--strict` to exit non-zero on warnings as well as errors:

```bash
# Validate Cooklang syntax, companion images, required metadata, emoji mappings, and ignored ingredients:
cook-docs -c ./cook --emoji-file ./includes/emoji.yaml --required-metadata source,serves doctor --check-recipes
# Run only syntax and ignored-ingredient checks in strict CI mode:
cook-docs -c ./cook --ignored-ingredients-file ./cook/config/ignored_ingredients.yaml doctor --check-recipes --only syntax,ignored --strict
# Validate specific recipe files in JSON mode:
cook-docs -c ./cook --required-metadata source doctor --check-recipes --json ./cook/breads/Bagels.cook
```

Recipe checks cover:

- **Cooklang syntax (`recipe_syntax`)**: strict parse errors (`[ERROR]`) and parser warnings (`[WARN]`)
- **Companion images (`recipe_images`)**: warns when no matching `.webp`, `.jpg`, `.jpeg`, or `.png` image exists alongside a recipe
- **Required metadata (`recipe_metadata`)**: errors when any key listed in `required_metadata` / `--required-metadata` is missing or empty
- **Unmapped emojis (`recipe_emojis`)**: warns about ingredients or cookware missing from `emoji_file` / inline `emoji` when configured
- **Ignored ingredients (`recipe_ignored`)**: warns when a `.cook` file tags an ingredient listed in `ignored_ingredients_file` or `ignored_ingredients` with `@`

Passing `--only` implies `--check-recipes`. Exits non-zero when errors are detected (or on any warning with `--strict`).

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

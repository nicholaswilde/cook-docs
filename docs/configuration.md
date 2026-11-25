# Configuration

`cook-docs` resolves settings in the following precedence order:

1. **CLI Flags**
2. **Environment Variables** (`COOK_DOCS_*`)
3. **Configuration File** (explicit `-C, --config` / `COOK_DOCS_CONFIG`, or auto-discovered)
4. **Hardcoded Defaults**

Here is a list of the supported parameters:

| Command Line                    | Environment Variable         | Config File      | Default            | Description                                                                                      |
|---------------------------------|------------------------------|------------------|--------------------|--------------------------------------------------------------------------------------------------|
| -C, --config path               | COOK_DOCS_CONFIG             | N/A              | N/A                | path to configuration file (TOML, YAML, or JSON)                                                 |
| -d, --dry-run                   | COOK_DOCS_DRY_RUN            | dry_run          | false              | don't actually render any markdown files just print to stdout                                    |
| -h, --help                      | N/A                          | N/A              | N/A                | help for cook-docs                                                                               |
| -i, --ignore-file string        | COOK_DOCS_IGNORE_FILE        | ignore_file      | .cookdocsignore    | filename to use as an ignore file to exclude recipe directories                                  |
| -j, --jsonify                   | COOK_DOCS_JSONIFY            | jsonify          | false              | parse recipes and print raw JSON to stdout (object for one recipe, array otherwise)              |
| -l, --log-level string          | COOK_DOCS_LOG_LEVEL          | log_level        | info               | level of logs that should be printed, one of (trace, debug, info, warn, warning, error)          |
| -o, --output-dir string         | COOK_DOCS_OUTPUT_DIR         | output_dir       | ""                 | custom output directory to write generated markdown documentation                                |
| --assets-dir string             | COOK_DOCS_ASSETS_DIR         | assets_dir       | ""                 | optional directory to copy companion recipe images into (slug-named)                             |
| --image-ext string              | COOK_DOCS_IMAGE_EXT          | image_ext        | ""                 | optional image extension override for `Info.ImageSlug` (e.g. `webp`)                             |
| --image-ext-map pairs           | COOK_DOCS_IMAGE_EXT_MAP      | image_ext_map    | `{}`               | per-extension `src=dst` mapping for `Info.ImageSlug` (e.g. `jpg=webp,jpeg=webp`)                 |
| --convert-images format         | COOK_DOCS_CONVERT_IMAGES     | convert_images   | ""                 | convert matching companion images into `--assets-dir` (`webp` via `cwebp`; `.png`/`.webp` kept)  |
| --image-quality int             | COOK_DOCS_IMAGE_QUALITY      | image_quality    | 80                 | output quality (`1`–`100`) for `--convert-images`                                                |
| --emoji-file path               | COOK_DOCS_EMOJI_FILE         | emoji_file       | ""                 | optional YAML/TOML/JSON file mapping ingredients, cookware, titles, and categories to emojis     |
| N/A                             | N/A                          | emoji            | N/A                | inline ingredient, cookware, title, and category emoji mappings in `.cookdocs.{yaml,toml,json}`  |
| --weights-file path             | COOK_DOCS_WEIGHTS_FILE       | weights_file     | ""                 | optional YAML/TOML/JSON ingredient density table for `toGrams` volume-to-weight conversion       |
| --link-dirs strings             | COOK_DOCS_LINK_DIRS          | link_dirs        | `[]`               | comma-separated recipe subdirectories eligible for `linkRecipe` cross-linking                    |
| --link-in-steps                 | COOK_DOCS_LINK_IN_STEPS      | link_in_steps    | false              | cross-link matching `@ingredient` AST references inside step directions                          |
| N/A                             | N/A                          | category_tags    | `{}`               | mapping of relative category directory paths to default tag lists (`Info.DefaultTags`/`Info.Tags`) |
| --required-metadata strings     | COOK_DOCS_REQUIRED_METADATA  | required_metadata | `[]`              | comma-separated metadata keys required by `cook-docs doctor --check-recipes`                     |
| --ignored-ingredients-file path | COOK_DOCS_IGNORED_INGREDIENTS_FILE | ignored_ingredients_file | ""         | optional YAML/TOML/JSON list of ingredients warned on when tagged with `@` in `doctor`           |
| N/A                             | COOK_DOCS_IGNORED_INGREDIENTS | ignored_ingredients | `[]`            | inline list of ignored ingredients (merged with `ignored_ingredients_file`)                      |
| --nav-file path                 | COOK_DOCS_NAV_FILE           | nav_file         | ""                 | optional Zensical (`zensical.toml`) or MkDocs (`mkdocs.yml`) nav file to sync rendered recipes into |
| --merge                         | COOK_DOCS_MERGE              | merge            | false              | non-destructively merge generated content into existing Markdown files                           |
| --force                         | COOK_DOCS_FORCE              | N/A              | false              | force full overwrite of existing Markdown files even when `--merge` / `merge` is enabled         |
| -c, --recipe-search-root string | COOK_DOCS_RECIPE_SEARCH_ROOT | recipe_search_root | .                | directory to search recursively within for recipes (and base root for mirrored output paths)     |
| -t, --template-files strings    | COOK_DOCS_TEMPLATE_FILES     | template_files   | `[recipe.md.gotmpl]` | gotemplate file paths (base names walk up parent dirs to `recipe_search_root`)                   |
| -w, --word-wrap int             | COOK_DOCS_WORD_WRAP          | word_wrap        | 120                | word wrap line length for recipe steps section                                                   |
| --watch                         | N/A                          | N/A              | false              | watch recipes and templates and re-render automatically on changes                               |
| --debounce MS                   | N/A                          | N/A              | 300                | debounce delay in milliseconds for watch mode                                                    |
| `[FILES]...`                    | N/A                          | N/A              | `[]`               | optional `.cook` files or subdirectories to render (mirrored relative to `recipe_search_root`)   |
| -v, --version                   | N/A                          | N/A              | N/A                | display the version of cook-docs                                                                 |

## Config Files

Configuration files may be used to set default application settings in **TOML**, **YAML**, or **JSON** format.
An example configuration file is provided in [`assets/config.example.toml`](https://github.com/nicholaswilde/cook-docs/blob/main/assets/config.example.toml) and bundled in release archives.

You can pass a configuration file explicitly using `-C, --config <path>` or the `COOK_DOCS_CONFIG` environment variable:

```bash
cook-docs -C path/to/config.toml
```

When no explicit config path is provided, `cook-docs` looks for `.cookdocs.toml`, `.cookdocs.yaml`, `.cookdocs.yml`, or `.cookdocs.json` in the following locations (in order):

- `/etc/cook-docs/`
- `~/.config/` (resolved via the OS user home directory: `$HOME` on Unix, `%USERPROFILE%` on Windows)
- `./`

```toml title=".cookdocs.toml"
assets_dir = ""
convert_images = ""
dry_run = false
emoji_file = ""
ignore_file = ".cookdocsignore"
ignored_ingredients = []
ignored_ingredients_file = ""
image_ext = ""
image_quality = 80
jsonify = false
link_dirs = []
link_in_steps = false
log_level = "info"
merge = false
nav_file = ""
output_dir = ""
recipe_search_root = "."
required_metadata = []
template_files = ["recipe.md.gotmpl"]
weights_file = ""
word_wrap = 120
```

```yaml title=".cookdocs.yaml"
---
assetsDir: ""
convertImages: ""
dryRun: false
emojiFile: ""
ignoreFile: .cookdocsignore
ignoredIngredients: []
ignoredIngredientsFile: ""
imageExt: ""
imageQuality: 80
jsonify: false
linkDirs: []
linkInSteps: false
logLevel: info
merge: false
navFile: ""
outputDir: ""
recipeSearchRoot: .
requiredMetadata: []
templateFiles:
  - recipe.md.gotmpl
weightsFile: ""
wordWrap: 120
```

```json title=".cookdocs.json"
{
  "assets_dir": "",
  "convert_images": "",
  "dry_run": false,
  "emoji_file": "",
  "ignore_file": ".cookdocsignore",
  "ignored_ingredients": [],
  "ignored_ingredients_file": "",
  "image_ext": "",
  "image_quality": 80,
  "jsonify": false,
  "link_dirs": [],
  "link_in_steps": false,
  "log_level": "info",
  "merge": false,
  "nav_file": "",
  "output_dir": "",
  "recipe_search_root": ".",
  "required_metadata": [],
  "template_files": ["recipe.md.gotmpl"],
  "weights_file": "",
  "word_wrap": 120
}
```

!!! note
    Keys in config files support `snake_case` (`dry_run`), `kebab-case` (`dry-run`), and `camelCase` (`dryRun`) formats.

## Emoji Mapping Schema (`emoji_file` / inline `emoji`)

Configure ingredient, cookware, title, and category fallback emojis either via `--emoji-file` (`emoji_file`) or inline under `emoji` in `.cookdocs.{yaml,toml,json}`. Both list-of-maps (YAML) and direct map (TOML/YAML/JSON) formats are supported:

```yaml title="emoji.yaml"
---
emoji:
  ingredients:
    - herb:
        - cilantro
        - parsley
  cookware:
    - bowl_with_spoon:
        - bowl
  titles:
    - pizza:
        - pizza
        - calzone
  categories:
    - bread:
        - breads
    - birthday:
        - desserts/cake
```

```toml title=".cookdocs.toml (inline)"
[emoji.ingredients]
herb = ["cilantro", "parsley"]

[emoji.cookware]
bowl_with_spoon = ["bowl"]

[emoji.titles]
pizza = ["pizza", "calzone"]

[emoji.categories]
breads = ":bread:"
desserts = "cake"
"desserts/cake" = ":birthday:"
```

ASCII shortcodes such as `herb` are formatted as `:herb:`; Unicode emojis such as `🌿` are emitted verbatim. Under `categories`, direct scalar maps use `category = emoji` (matching relative `.Info.Categories` paths from most-specific path to parent fallback), while list-of-maps and array values use `emoji = ["category", ...]`.

## Ingredient Density Schema (`weights_file`)

Configure volume-to-weight conversions for `toGrams` in YAML, TOML, or JSON. Plain numeric values specify grams per US cup; structured objects support `grams_per_cup`, `grams_per_ml` (`density`), or `grams` + `volume` (e.g. `"1 tbsp"`) along with optional `aliases`. Optional `exclude` and `exclude_small_units` lists skip conversion for specific ingredients (or for `tsp`/`tbsp` amounts of spices/seasonings). Optional `exclude_small_units_except` (`exclude-small-units-except`, `excludeSmallUnitsExcept`) lists base-name tokens that bypass `exclude_small_units` (e.g. allowing `basil pesto` or `cinnamon sugar` to convert while keeping `fresh basil` or `ground cinnamon` excluded; explicit `exclude` still applies):

```yaml title="weights.yaml"
---
exclude_small_units:
  - salt
  - basil
  - cinnamon
exclude_small_units_except:
  - pesto
  - oil
  - sugar
weights:
  all-purpose flour:
    grams_per_cup: 120
    aliases:
      - ap flour
      - flour
  basil pesto: 224
  granulated sugar: 200
  olive oil:
    grams_per_ml: 0.915
  honey:
    grams: 21
    volume: "1 tbsp"
```

## Category Default Tags (`category_tags`)

Map relative recipe directory paths (e.g. `breads`, `desserts/cake`, `cookies-and-bars`) to default tag lists in `.cookdocs.{toml,yaml,json}` (`category_tags`, `category-tags`, or `categoryTags`). `cook-docs` resolves `.Info.DefaultTags` via longest-prefix match against `.Info.Categories` and merges them with `.Frontmatter.tags` (case-insensitively deduplicated) into `.Info.Tags`:

```toml title=".cookdocs.toml"
[category_tags]
beverages = ["beverage"]
breads = ["bread"]
cookies-and-bars = ["cookie", "bar", "dessert"]
desserts = ["dessert"]
"desserts/cake" = ["dessert", "cake"]
```

## Ignored Ingredients (`ignored_ingredients_file` / `ignored_ingredients`)

Configure ingredients that should not be tagged with `@` in `.cook` recipes so `cook-docs doctor --check-recipes` (or `--only ignored`) emits a warning when they appear. Both Cooklang's top-level YAML list (`config/ignored_ingredients.yaml`) and TOML/JSON list or keyed object (`ignored_ingredients`) formats are supported:

```yaml title="config/ignored_ingredients.yaml"
---
- nonstick baking spray
```

```toml title=".cookdocs.toml (inline)"
ignored_ingredients = ["nonstick baking spray"]
```

## Navigation Sync (`nav_file`)

Set `nav_file` (or pass `--nav-file <PATH>` / `COOK_DOCS_NAV_FILE`) to a Zensical (`zensical.toml`) or MkDocs (`mkdocs.yml` / `mkdocs.yaml`) config file to automatically insert, deduplicate, and alphabetically maintain rendered recipes in the matching category or subcategory navigation list while preserving surrounding formatting, comments, and YAML tags:

```toml title=".cookdocs.toml"
output_dir = "docs"
nav_file = "zensical.toml"
```

- **Zensical (`zensical.toml`)**: supports both `[[project.nav]]` array-of-tables and `[project] nav = [...]` inline arrays, including nested subcategories (e.g. `Ingredients` > `Butter`) and flattened parent categories (e.g. `desserts/cake/*.md` inside `Desserts`).
- **MkDocs (`mkdocs.yml` / `mkdocs.yaml`)**: updates only the top-level `nav:` block so comments, blank lines, and custom tags such as `!!python/name:...` outside `nav:` remain untouched.
- **Dry-run safety**: when combined with `--dry-run`, planned additions, updates, and deduplications are printed to stdout without modifying `nav_file`.
- **Doctor checks**: `cook-docs doctor` validates that a configured `nav_file` exists and parses cleanly.

## Environment Variables

Environment variables are also supported. They start with the prefix `COOK_DOCS_` and use
underscores instead of dashes (e.g., `COOK_DOCS_DRY_RUN`, `COOK_DOCS_CONFIG`).
Comma-separated values are supported for `COOK_DOCS_TEMPLATE_FILES`, `COOK_DOCS_LINK_DIRS`, `COOK_DOCS_REQUIRED_METADATA`, and `COOK_DOCS_IGNORED_INGREDIENTS`.

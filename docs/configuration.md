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
| --emoji-file path               | COOK_DOCS_EMOJI_FILE         | emoji_file       | ""                 | optional YAML/TOML/JSON file mapping ingredients, cookware, titles, and categories to emojis     |
| N/A                             | N/A                          | emoji            | N/A                | inline ingredient, cookware, title, and category emoji mappings in `.cookdocs.{yaml,toml,json}`  |
| --weights-file path             | COOK_DOCS_WEIGHTS_FILE       | weights_file     | ""                 | optional YAML/TOML/JSON ingredient density table for `toGrams` volume-to-weight conversion       |
| --link-dirs strings             | COOK_DOCS_LINK_DIRS          | link_dirs        | `[]`               | comma-separated recipe subdirectories eligible for `linkRecipe` cross-linking                    |
| --required-metadata strings     | COOK_DOCS_REQUIRED_METADATA  | required_metadata | `[]`              | comma-separated metadata keys required by `cook-docs doctor --check-recipes`                     |
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
- `~/.config/`
- `./`

```toml title=".cookdocs.toml"
assets_dir = ""
dry_run = false
emoji_file = ""
ignore_file = ".cookdocsignore"
image_ext = ""
jsonify = false
link_dirs = []
log_level = "info"
merge = false
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
dryRun: false
emojiFile: ""
ignoreFile: .cookdocsignore
imageExt: ""
jsonify: false
linkDirs: []
logLevel: info
merge: false
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
  "dry_run": false,
  "emoji_file": "",
  "ignore_file": ".cookdocsignore",
  "image_ext": "",
  "jsonify": false,
  "link_dirs": [],
  "log_level": "info",
  "merge": false,
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

## Environment Variables

Environment variables are also supported. They start with the prefix `COOK_DOCS_` and use
underscores instead of dashes (e.g., `COOK_DOCS_DRY_RUN`, `COOK_DOCS_CONFIG`).
Comma-separated values are supported for `COOK_DOCS_TEMPLATE_FILES`, `COOK_DOCS_LINK_DIRS`, and `COOK_DOCS_REQUIRED_METADATA`.

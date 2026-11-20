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
| -j, --jsonify                   | COOK_DOCS_JSONIFY            | jsonify          | false              | parse the recipe and display it in json format                                                   |
| -l, --log-level string          | COOK_DOCS_LOG_LEVEL          | log_level        | info               | level of logs that should be printed, one of (trace, debug, info, warn, warning, error)          |
| -o, --output-dir string         | COOK_DOCS_OUTPUT_DIR         | output_dir       | ""                 | custom output directory to write generated markdown documentation                                |
| -c, --recipe-search-root string | COOK_DOCS_RECIPE_SEARCH_ROOT | recipe_search_root | .                | directory to search recursively within for recipes                                               |
| -t, --template-files strings    | COOK_DOCS_TEMPLATE_FILES     | template_files   | [recipe.md.gotmpl] | gotemplate file paths relative to each recipe directory from which documentation will be generated |
| -w, --word-wrap int             | COOK_DOCS_WORD_WRAP          | word_wrap        | 120                | word wrap line length for recipe steps section                                                   |
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
dry_run = false
ignore_file = ".cookdocsignore"
jsonify = false
log_level = "info"
output_dir = ""
recipe_search_root = "."
template_files = ["recipe.md.gotmpl"]
word_wrap = 120
```

```yaml title=".cookdocs.yaml"
---
dryRun: false
ignoreFile: .cookdocsignore
jsonify: false
logLevel: info
outputDir: ""
recipeSearchRoot: .
templateFiles:
  - recipe.md.gotmpl
wordWrap: 120
```

```json title=".cookdocs.json"
{
  "dry_run": false,
  "ignore_file": ".cookdocsignore",
  "jsonify": false,
  "log_level": "info",
  "output_dir": "",
  "recipe_search_root": ".",
  "template_files": ["recipe.md.gotmpl"],
  "word_wrap": 120
}
```

!!! note
    Keys in config files support `snake_case` (`dry_run`), `kebab-case` (`dry-run`), and `camelCase` (`dryRun`) formats.

## Environment Variables

Environment variables are also supported. They start with the prefix `COOK_DOCS_` and use
underscores instead of dashes (e.g., `COOK_DOCS_DRY_RUN`, `COOK_DOCS_CONFIG`).
Comma-separated values are supported for `COOK_DOCS_TEMPLATE_FILES`.

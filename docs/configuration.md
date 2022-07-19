# Configuration

`cook-docs` uses [spf13's][1] [viper][2] configuration library to handle application settings.

Here is a list of the current supported parameters:

| Command Line                    | Environmental Variable        | Config File       | Default             | Description                                                                                         |
|---------------------------------|-------------------------------|-------------------|---------------------|-----------------------------------------------------------------------------------------------------|
| -d, --dry-run                   | COOK_DOCS_DRY_RUN             | dryRun            | false               | don't actually render any markdown files just print to stdout passed                                |
| -h, --help                      | N/A                           | N/A               | N/A                 | help for cook-docs                                                                                  |
| -i, --ignore-file string        | COOK_DOCS_IGNORE_FILE         | ignoreFile        | .cookdocsignore     | filename to use as an ignore file to exclude recipe directories                                     |
| -j, --jsonify                   | COOK_DOCS_JSONIFY             | jsonify           | false               | parse the recipe and display it in json format                                                      |
| -l, --log-level string          | COOK_DOCS_LOG_LEVEL           | logLevel          | info                | level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace)        |
| -c, --recipe-search-root string | COOK_DOCS_RECIPE_SEARCH_ROOT  | recipeSearchRoot  | .                   | directory to search recursively within for recipes.                                                 |
| -t, --template-files strings    | COOK_DOCS_TEMPLATE_FILES      | templateFiles     | [recipe.md.gotmpl]  | gotemplate file paths relative to each recipe directory from which documentation will be generated  |
| -w, --word-wrap int             | COOK_DOCS_WORD_WRAP           | wordWrap          | 120                 | word wrap line length for recipe steps section                                                      |
| -v, --version                   | N/A                           | N/A               | N/A                 | diplay the version of cook-docs                                                                     |

## Config Files

Configuration files may be used to set the default app settings.

The config file name is `.cookdocs`, `.cookdocs.yaml` or `.cookdocs.yml` and can be located in
any of the following locations:

- `/etc/cook-docs/`
- `~/.config/`
- `./`

```yaml title=".cookdocs.yaml"
---
dryRun: false
ignoreFile: .cookdocsignore
jsonify: false
logLevel: info
templateFiles:
  - recipe.md.gotmpl
wordWrap: 120
```

!!! note
    The variables in the config file can be both in the `Command Line` or `Config File` format. E.g. `dry-run` and `dryRun`.

## Environmental Variables

Environmental variables are also supported. They start with the prefix `COOK_DOCS_` and use
underscores instead of dashes.

[1]: https://github.com/spf13
[2]: https://github.com/spf13/viper
[3]: https://github.com/spf13/viper#reading-config-files

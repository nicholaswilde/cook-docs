# Usage

!!! warning
      The mode of operation of `cook-docs` is to process all recipes in the
      working directory and sub folders. See [Mode of Operation][3] for
      details.

There are two important parameters to be aware of when running `cook-docs`.
`--recipe-search-root` specifies the directory under which the tool will
recursively search for recipes to render documentation for.
`--template-files` specifies the list of gotemplate files that should be used
in rendering the resulting markdown file for each chart found. By default
`--recipe-search-root=.` and `--template-files=recipe.md.gotmpl`.

If a template file is specified as a filename only as with the default above,
the file is interpreted as being _relative to each chart directory found_. If,
however, a template file is specified as a relative path, e.g. the first of
`--template-files=./_templates.gotmpl --template-files=recipe.md.gotmpl` then
the file is interpreted as being relative to the `recipe-search-root`.

## :runner: Running the Binary Directly

To run and generate documentation into markdown files for all cooklang recipes within or recursively contained by a directory:

```bash
cook-docs
# OR
cook-docs --dry-run # prints generated documentation to stdout rather than modifying markdown files.
```

The tool searches recursively through subdirectories of the current directory for `<Recipe Name>.cook` files and generates documentation
for every recipe that it finds.


## :file_folder: Ignoring Recipe Directories

cook-docs supports a `.cookdocsignore` file, exactly like a `.gitignore` file in which one can specify directories to ignore
when searching for recipes. Directories specified need not be charts themselves, so parent directories containing potentially
many recipes can be ignored and none of the recipes underneath them will be processed. You may also directly reference the
`<Recipe Name>.cook` file for a chart to skip processing for it.


## :grey_question: Help

Use the `--help` parameter for help on using `cook-docs`.

```bash
cook-docs --help
```

## :robot: Task

[go-task][1] may be used to automate `cook-docs` tasks.

Run `task` for a list of tasks:

```bash
task
```

## :material-git: pre-commit

[pre-commit][2] may also be used on this project.

```bash
pre-commit install
pre-commit install-hooks
```

[1]: https://taskfile.dev/#/
[2]: https://pre-commit.com/
[3]: ./about#mode-of-operation

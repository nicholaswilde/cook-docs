# Usage

## Running the binary directly

To run and generate documentation into markdown files for all cooklang recipes within or recursively contained by a directory:

```bash
cook-docs
# OR
cook-docs --dry-run # prints generated documentation to stdout rather than modifying markdown files.
```

The tool searches recursively through subdirectories of the current directory for `<Recipe name>.cook` files and generates documentation
for every recipe that it finds.

## Ignoring Recipe Directories

cook-docs supports a `.cookdocsignore` file, exactly like a `.gitignore` file in which one can specify directories to ignore
when searching for recipes. Directories specified need not be charts themselves, so parent directories containing potentially
many recipes can be ignored and none of the recipes underneath them will be processed. You may also directly reference the
`<Recipe Name>.cook` file for a chart to skip processing for it.

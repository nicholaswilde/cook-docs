# Templates

There are two important parameters to be aware of when running cook-docs. `--recipe-search-root` specifies the directory
under which the tool will recursively search for recipes to render documentation for. `--template-files` specifies the list
of gotemplate files that should be used in rendering the resulting markdown file for each chart found. By default
`--recipe-search-root=.` and `--template-files=recipe.md.gotmpl`.

If a template file is specified as a filename only as with the default above, the file is interpreted as being _relative to each chart directory found_.
If, however, a template file is specified as a relative path, e.g. the first of `--template-files=./_templates.gotmpl --template-files=recipe.md.gotmpl`
then the file is interpreted as being relative to the `recipe-search-root`.

If any of the specified template files is not found for a recipe (you'll notice most of the example recipe do not have a recipe.md.gotmpl)
file, then the internal default template is used instead.

The default internal template mentioned above uses many of these and looks like this:

```title="recipe.md.gotmpl"
{{ template "cook.headerSection" . }}

{{ template "cook.imageSection" . }}

{{ template "cook.tableSection" . }}

{{ template "cook.ingredientsSection" . }}

{{ template "cook.cookwareSection" . }}

{{ template "cook.stepsSection" . }}

{{ template "cook.sourceSection" . }}
```

The tool also includes the [sprig templating library](https://github.com/Masterminds/sprig), so those functions can be used
in the templates you supply.

## Custom Sections

Custom sections may be specified in the template by using the `define` parameter.

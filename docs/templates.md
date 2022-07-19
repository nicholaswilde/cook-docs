# Templates

The default template may be overwritten adding `recipe.md.gotmpl` files to the
recipe directories.

If any of the specified template files is not found for a recipe (you'll notice
most of the example recipe do not have a `recipe.md.gotmpl`) file, then the
internal default template is used instead.

The default internal template mentioned above uses many of these and looks like
this:

```go title="recipe.md.gotmpl"
{{ template "cook.headerSection" . }}

{{ template "cook.imageSection" . }}

{{ template "cook.tableSection" . }}

{{ template "cook.ingredientsSection" . }}

{{ template "cook.cookwareSection" . }}

{{ template "cook.stepsSection" . -}}

{{ template "cook.sourceSection" . }}
```

The tool also includes the [sprig templating library][5], so those functions
can be used in the templates you supply.

## Built-in Templates

### Sections

| Name                                     | Description                                                        |
|------------------------------------------|--------------------------------------------------------------------|
| `cook.headerSection`                     | The main heading of the generated markdown file                    |
| `cook.imageSection`                      | The image section                                                  |
| `cook.tableSection`                      | The table section that consists of the serving size and total time |
| `cook.ingredientsSection`                | The ingredients section                                            |
| `cook.cookwareSection`                   | The cookware section                                               |
| `cook.stepsSection`                      | The steps section                                                  |
| `cook.stepsWithQuotedCommentsSection`    | The steps section with comments in [block quotes][8]               |
| `cook.stepsWithAdmonishedCommentsHeader` | The steps section with comments in [admonitions][9]                |
| `cook.sourceSection`                     | The source section if `source` exists in Metadata                  |
| `cook.commentsSection`                   | The comments section                                               |
| `cook.metadataSection`                   | The metadata section. This prints all values in `Metadata`         |

### Components

| Name                                     | Description                                                                  |
|------------------------------------------|------------------------------------------------------------------------------|
| `cook.ingredientsHeader`                 | The ingredients header                                                       |
| `cook.ingredients`                       | An unordered list of the ingredients                                         |
| `cook.cookwareHeader`                    | The cookware header                                                          |
| `cook.cookware`                          | An unordered list of cookware                                                |
| `cook.stepsHeader`                       | The steps header                                                             |
| `cook.steps`                             | A list of steps. Each step has its own sub heading labeled as `Step #`       |
| `cook.stepsWithQuotedCommentsHeader`     | The steps with block quotes header                                           |
| `cook.stepsWithQuotedComments`           | A list of steps with block quoted comments in between                        |
| `cook.stepsWithAdmonishedCommentsHeader` | The steps with admonitions headder                                           |
| `cook.stepsWithAdmonishedComments`       | A list of steps with comments as admonitions in between                      |
| `cook.sourceHeader`                      | Source header                                                                |
| `cook.source`                            | The `source` as a single unordered list item                                 |
| `cook.metadataHeader`                    | The `Metadata` header                                                        |
| `cook.metadata`                          | An unordered list of the `Metadata`.                                         |
| `cook.commentsHeader`                    | The comments header                                                          |
| `cook.comments`                          | An unordered list of the comments                                            |
| `.Info.RecipeName`                       | The name of the recipe taken fromt the recipe file name                      |
| `.Info.ImageFileName`                    | The image name if an image file is found                                     |
| `.Info.ImageFilePath`                    | The image path if an image file is found                                     |
| `.Info.NewRecipeFilePath`                | The new recipe file name after removal of spaces and converting to lowercase |
| `.Info.RecipeFilePath`                   | The file path of the recipe file                                             |

See [template.go][6] for how each key is defined.

!!! note
      The `cook.commentsSection` does not print the comments properly [(#3)][7]

## Metadata

Any [metadata][10] from the recipe `*.cook` file will be written to the `cook.metadataSection`.

```
>> source: https://www.gimmesomeoven.com/baked-potato/
>> time required: 1.5 hours
>> course: dinner
...
```

The names of the markdown and image files are made lowercase and the spaces are replaced
by dashes. E.g. `My Recipe Name.cook -> my-recipe-name.md` and `My Recipe Name.png -> my-recipe-name.png`.

## Custom Sections

Custom sections may be specified in the template by using the `define` parameter.

```go title="recipe.md.gotmpl"
{{- define "custom.section" -}}
# My custom section
{{- end -}}
...
```

Then use it later in the template.

```go title="recipe.md.gotmpl"
...
{{ template "custom.section" . }}
```

```go title="Output"
# My custom section
```

## Cooklang Parser

`cook-docs` uses [aquilax's][1] [cooklang-go][2] parser to parse `cooklang`
recipes. The recipes are then merged with custom cook-docs data, such as
`Info` and `Config`. The data output may then be directly used inside of
the `cook-docs` template files.

See [`parser.go`][3] for the basic structure latyout.

The `jsonify` option may also be used to output

```json title="Example parsed `cook-docs` output"
 Output:
 {
   "Steps": [
     {
       "Directions": "Make 6 pizza balls using tipo zero flour, water, salt and fresh yeast. Put in a fridge for 2 days.",
       "Timers": [
         {
           "Name": "",
           "Duration": 2,
           "Unit": "days"
         }
       ],
       "Ingredients": [
         {
           "Name": "tipo zero flour",
           "Amount": {
             "IsNumeric": true,
             "Quantity": 820,
             "QuantityRaw": "820",
             "Unit": "g"
           }
         },
         {
           "Name": "water",
           "Amount": {
             "IsNumeric": true,
             "Quantity": 533,
             "QuantityRaw": "533",
             "Unit": "ml"
           }
         },
         {
           "Name": "salt",
           "Amount": {
             "IsNumeric": true,
             "Quantity": 24.6,
             "QuantityRaw": "24.6",
             "Unit": "g"
           }
         },
         {
           "Name": "fresh yeast",
           "Amount": {
             "IsNumeric": true,
             "Quantity": 1.6,
             "QuantityRaw": "1.6",
             "Unit": "g"
           }
         }
       ],
       "Cookware": [
         {
           "IsNumeric": false,
           "Name": "fridge",
           "Quantity": 1,
           "QuantityRaw": ""
         }
       ],
       "Comments": null
     },
     {
       "Directions": "Set oven to max temperature and heat pizza stone for about 40 minutes.",
       "Timers": [
         {
           "Name": "",
           "Duration": 40,
           "Unit": "minutes"
         }
       ],
       "Ingredients": [],
       "Cookware": [
         {
           "IsNumeric": false,
           "Name": "oven",
           "Quantity": 1,
           "QuantityRaw": ""
         },
         {
           "IsNumeric": false,
           "Name": "pizza stone",
           "Quantity": 1,
           "QuantityRaw": ""
         }
       ],
       "Comments": null
     },
     {
       "Directions": "Make some tomato sauce with chopped tomato and garlic and dried oregano. Put on a pan and leave for 15 minutes occasionally stirring.",
       "Timers": [
         {
           "Name": "",
           "Duration": 15,
           "Unit": "minutes"
         }
       ],
       "Ingredients": [
         {
           "Name": "chopped tomato",
           "Amount": {
             "IsNumeric": true,
             "Quantity": 3,
             "QuantityRaw": "3",
             "Unit": "cans"
           }
         },
         {
           "Name": "garlic",
           "Amount": {
             "IsNumeric": true,
             "Quantity": 3,
             "QuantityRaw": "3",
             "Unit": "cloves"
           }
         },
         {
           "Name": "dried oregano",
           "Amount": {
             "IsNumeric": true,
             "Quantity": 3,
             "QuantityRaw": "3",
             "Unit": "tbsp"
           }
         }
       ],
       "Cookware": [
         {
           "IsNumeric": false,
           "Name": "pan",
           "Quantity": 1,
           "QuantityRaw": ""
         }
       ],
       "Comments": null
     },
     {
       "Directions": "Make pizzas putting some tomato sauce with spoon on top of flattened dough. Add fresh basil, parma ham and mozzarella.",
       "Timers": [],
       "Ingredients": [
         {
           "Name": "fresh basil",
           "Amount": {
             "IsNumeric": true,
             "Quantity": 18,
             "QuantityRaw": "18",
             "Unit": "leaves"
           }
         },
         {
           "Name": "parma ham",
           "Amount": {
             "IsNumeric": true,
             "Quantity": 3,
             "QuantityRaw": "3",
             "Unit": "packs"
           }
         },
         {
           "Name": "mozzarella",
           "Amount": {
             "IsNumeric": true,
             "Quantity": 3,
             "QuantityRaw": "3",
             "Unit": "packs"
           }
         }
       ],
       "Cookware": [
         {
           "IsNumeric": false,
           "Name": "spoon",
           "Quantity": 1,
           "QuantityRaw": ""
         }
       ],
       "Comments": null
     },
     {
       "Directions": "Put in an oven for 4 minutes.",
       "Timers": [
         {
           "Name": "",
           "Duration": 4,
           "Unit": "minutes"
         }
       ],
       "Ingredients": [],
       "Cookware": [
         {
           "IsNumeric": false,
           "Name": "oven",
           "Quantity": 1,
           "QuantityRaw": ""
         }
       ],
       "Comments": null
     }
   ],
   "Metadata": {
     "servings": "6",
     "source": "https://www.somewebsite.com/pizza-balls"
   },
   "Config": {
     "DryRun": false,
     "Jsonify": true,
     "IgnoreFile": ".cookdocsignore",
     "RecipeSearchRoot": ".",
     "LogLevel": "info",
     "TemplateFiles": [
       "recipe.md.gotmpl"
     ],
     "WordWrap": 120
   },
   "Info": {
     "ImageFilePath": "/home/nicholas/git/nicholaswilde/cook-docs/cmd/cook-docs/testdata/My Test Recipe.png",
     "ImageFileName": "My Test Recipe.png",
     "RecipeName": "My Test Recipe",
     "RecipeFilePath": "/home/nicholas/git/nicholaswilde/cook-docs/cmd/cook-docs/testdata/My Test Recipe.cook",
     "NewRecipeFilePath": "/home/nicholas/git/nicholaswilde/cook-docs/cmd/cook-docs/testdata/my-test-recipe.md"
   }
 }
}
```

## Spacing

Spacing for the templates is controlled by the minus signs inside of the
delimiters. See [Text and spaces][4].

```go title="example"
{{- define "custom.section" . -}}
{{- end -}}
```

!!! note
    To remove the double EOF new lines when `.Metadata.source` is missing from the recipe file but present in the template file, double new lines is added to the beginning of `cook.sourceSection` and white space is removed from the end
    of `cook.stepsSection`.

[1]: https://github.com/aquilax
[2]: https://github.com/aquilax/cooklang-go
[3]: https://github.com/aquilax/cooklang-go/blob/490a595d639b679a4f2053a309647882db37e569/parser.go
[4]: https://pkg.go.dev/text/template#hdr-Text_and_spaces
[5]: https://github.com/Masterminds/sprig
[6]: https://github.com/nicholaswilde/cook-docs/blob/main/pkg/document/template.go
[7]: https://github.com/nicholaswilde/cook-docs/issues/3
[8]: https://github.github.com/gfm/#block-quotes
[9]: https://squidfunk.github.io/mkdocs-material/reference/admonitions/
[10]: https://cooklang.org/docs/spec/#metadata

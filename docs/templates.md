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

{{ template "cook.stepsSection" . }}

{{ template "cook.sourceSection" . }}
```

The tool also includes the [sprig templating library][5], so those functions
can be used in the templates you supply.

## Built-in Templates

### Sections

| Name                    | Description                                                        |
|-------------------------|--------------------------------------------------------------------|
| cook.headerSection      | The main heading of the generated markdown file                    |
| cook.imageSection       | The image section                                                  |
| cook.tableSection       | The table section that consists of the serving size and total time |
| cook.ingredientsSection | The ingredients section                                            |
| cook.cookwareSection    | The cookware section                                               |
| cook.stepsSection       | The steps section                                                  |
| cook.sourceSection      | The source section if `source` exists in Metadata                  |
| cook.commentsSection    | The comments section                                               |
| cook.metadataSection    | The metadata section. This prints all values in `Metadata`         |

### Components

| Name                   | Description                                                               |
|------------------------|---------------------------------------------------------------------------|
| cook.ingredientsHeader | The ingredients header                                                    |
| cook.ingredients       | An unordered list of the ingredients                                      |
| cook.cookwareHeader    | The cookware header                                                       |
| cook.cookware          | An unordered list of cookware                                             |
| cook.stepsHeader       | The steps header                                                          |
| cook.steps						 | A list of steps. Each step has its own sub heading labeled as `Step #`    |
| cook.sourceHeader      | Source header                                                             |
| cook.source            | The `source` as a single unordered list item                              |
| cook.metadataHeader    | `Metadata` header                                                         |
| cook.metadata          | An unordered list of the `Metadata`. `ImageName` and `title` are included |
| cook.commentsHeader    | The comments header                                                       |
| cook.comments          | An unordered list of the comments                                         |

See [template.go][6] for how each key is defined.

!!! note
      The `cook.commentsSection` does not print the comments properly [(#3)][7]

## Custom Sections

Custom sections may be specified in the template by using the `define` parameter.

```go
{{- define "custom.section" -}}
# My custom section
{{- end -}}
...
```

Then use it later in the template.

```go
...
{{ template "custom.section" . }}
```

```go title="Output"
# My custom section
```

## Cooklang Parser

`cook-docs` uses [aquilax's][1] [cooklang-go][2] parser to parse `cooklang`
recipes. The data output may then be directly used inside of the `cook-docs`
template files.

See [`parser.go`][3] for the structure latyout.

```json title="Example parsed output"
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
	          "Name": "fridge"
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
	          "Name": "oven"
	        },
	        {
	          "Name": "pizza stone"
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
	          "Name": "pan"
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
	          "Name": "spoon"
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
	          "Name": "oven"
	        }
	      ],
	      "Comments": null
	    }
	  ],
	  "Metadata": {
	    "servings": "6"
	  }
	}
```

## Metadata

`cook-docs` uses the `Metadata.title` and `Metadata.ImageName` keys for the
recipe title, taken from the `*.cook` filename and name of the formatted image
name. If the parsed recipe uses these keys, they will be overwritten by
`cook-docs`.

```title="Overwritten Recipe Metadata"
>> title: My recipe title
>> ImageName: My image name
...
```

The names of the markdown and image files are made lowercase and the spaces are replaced
by dashes. E.g. `My Recipe Name.cook -> my-recipe-name.md`

## Spacing

Spacing for the templates is controlled by the minus signs inside of the
delimiters. See [Text and spaces][4].

[1]: https://github.com/aquilax
[2]: https://github.com/aquilax/cooklang-go
[3]: https://github.com/aquilax/cooklang-go/blob/490a595d639b679a4f2053a309647882db37e569/parser.go
[4]: https://pkg.go.dev/text/template#hdr-Text_and_spaces
[5]: https://github.com/Masterminds/sprig
[6]: https://github.com/nicholaswilde/cook-docs/blob/main/pkg/document/template.go
[7]: https://github.com/nicholaswilde/cook-docs/issues/3

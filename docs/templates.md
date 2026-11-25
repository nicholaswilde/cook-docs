# Templates

The default template may be overridden by placing a `recipe.md.gotmpl` file in a
recipe directory or any parent directory up to `--recipe-search-root`. A single
root `recipe.md.gotmpl` covers all nested subdirectories, while a closer template
in a subdirectory overrides it locally.

If none of the specified template files is found between the recipe directory and
`--recipe-search-root`, the internal default template is used instead.

The default internal template mentioned above uses many of these and looks like
this:

```go title="recipe.md.gotmpl"
{{ template "cook.headerSection" . }}

{{ template "cook.lazyImageSection" . }}

{{ template "cook.tableSection" . }}

{{ template "cook.ingredientsSection" . }}

{{ template "cook.cookwareSection" . }}

{{ template "cook.stepsSection" . -}}

{{ template "cook.sourceSection" . }}
```

The tool renders Go-compatible templates using [`gtmpl-ng`][5] and provides built-in helper functions (`getSource`, `sumTimers`, `totalTime`, `toYaml`, `emoji`, `ingredientEmoji`, `cookwareEmoji`, `titleEmoji`, `toGrams`, `linkRecipe`, `scale`, `lower`, `replace`, `round`, `add1`, `wrap`, `indent`) for use in custom templates.

## Built-in Templates

### Sections

| Name                                      | Description                                                        |
|-------------------------------------------|--------------------------------------------------------------------|
| `cook.headerSection`                      | The main heading (`Metadata.title` when present, otherwise `Info.RecipeName`), optionally prefixed by `titleEmoji` when emoji mappings are configured |
| `cook.imageSection`                       | The image section                                                               |
| `cook.lazyImageSection`                   | The lazy-loaded image section (`loading="lazy"`)                                |
| `cook.tableSection`                       | Servings and total time (`totalTime`: prefers `total time` metadata over timers) |
| `cook.ingredientsSection`                 | The ingredients section                                            |
| `cook.cookwareSection`                    | The cookware section (omitted automatically when `.Cookware` is empty) |
| `cook.stepsSection`                       | The steps section                                                  |
| `cook.stepsWithQuotedCommentsSection`     | The steps section with comments in [block quotes][8]               |
| `cook.stepsWithAdmonishedCommentsSection` | The steps section with comments in [admonitions][9]                |
| `cook.sourceSection`                      | The source section if `source` exists in Metadata                  |
| `cook.commentsSection`                    | The comments section                                               |
| `cook.metadataSection`                    | The metadata section. This prints all values in `Metadata`         |

### Components

| Name                                     | Description                                                                  |
|------------------------------------------|------------------------------------------------------------------------------|
| `cook.ingredientsHeader`                 | The ingredients header                                                       |
| `cook.ingredients`                       | Unordered list of aggregated ingredients (preserving fractions, formatting non-numeric quantities cleanly, omitting `"some "` on bare items, with optional emojis, gram weights, and cross-links) |
| `cook.cookwareHeader`                    | The cookware header                                                          |
| `cook.cookware`                          | Unordered list of deduplicated cookware (with optional emojis)               |
| `cook.stepsHeader`                       | The steps header                                                             |
| `cook.steps`                             | A list of steps. Each step has its own sub heading labeled as `Step #`       |
| `cook.stepsWithQuotedCommentsHeader`     | The steps with block quotes header                                           |
| `cook.stepsWithQuotedComments`           | A list of steps with block quoted comments in between                        |
| `cook.stepsWithAdmonishedCommentsHeader` | The steps with admonitions header                                            |
| `cook.stepsWithAdmonishedComments`       | A list of steps with comments as admonitions in between                      |
| `cook.sourceHeader`                      | Source header                                                                |
| `cook.source`                            | The `source` as a single unordered list item                                 |
| `cook.metadataHeader`                    | The `Metadata` header                                                        |
| `cook.metadata`                          | An unordered list of the `Metadata`.                                         |
| `cook.commentsHeader`                    | The comments header                                                          |
| `cook.comments`                          | An unordered list of the comments                                            |
| `.Info.RecipeName`                       | The name of the recipe taken from the recipe file name                       |
| `.Info.RecipeSlug`                       | The lowercased hyphenated recipe stem (e.g. `my-test-recipe`)                |
| `.Info.ImageFileName`                    | The image name if an image file is found                                     |
| `.Info.ImageFilePath`                    | The image path if an image file is found                                     |
| `.Info.ImageSlug`                        | The lowercased hyphenated image filename (with optional `--image-ext` override) |
| `.Info.NewRecipeFilePath`                | The new recipe file name after removal of spaces and converting to lowercase |
| `.Info.RecipeFilePath`                   | The file path of the recipe file                                             |
| `.Info.RelRoot`                          | Relative path back to `--recipe-search-root` (`.`, `..`, `../..`, ...)       |
| `.Info.Categories`                       | Relative directory segments under `--recipe-search-root` (e.g. `["desserts", "cake"]`) |
| `.Info.DefaultTags`                      | Default tags resolved from `category_tags` for `.Info.Categories` (longest-prefix match) |
| `.Info.Tags`                             | Deduplicated union of `.Info.DefaultTags` and `.Frontmatter.tags`            |
| `.Frontmatter`                           | Structured YAML/Cooklang metadata preserving lists, booleans, numbers, and maps |
| `.Ingredients`                           | Aggregated top-level recipe ingredients (deduplicated and summed across steps) |
| `.Cookware`                              | Aggregated top-level recipe cookware (deduplicated across steps)             |

### Helper Functions

| Function    | Description                                                                   |
|-------------|-------------------------------------------------------------------------------|
| `getSource` | Formats a `source` metadata URL into a Markdown link                          |
| `sumTimers` | Sums recipe step timers into a total duration string                          |
| `totalTime` | Uses `total time` metadata when present, otherwise falls back to `sumTimers`  |
| `toYaml`    | Formats a structured value (list, map, or scalar) as deterministic YAML       |
| `emoji`     | Looks up an emoji (`emoji "ingredient" .Name`, `emoji "cookware" .Name`, `emoji "title" .Info.RecipeName`, `emoji "category" "breads"`, or `emoji .Name`) |
| `ingredientEmoji` | Looks up an ingredient emoji shortcode or Unicode glyph (or `""` if unmapped) |
| `cookwareEmoji` | Looks up a cookware emoji shortcode or Unicode glyph (or `""` if unmapped)  |
| `titleEmoji` | Resolves a recipe header emoji from title mappings or `[emoji.categories]` fallback (`titleEmoji .`, `titleEmoji .Info.RecipeName`, or `titleEmoji .Info.RecipeName .Info.Categories`); returns `""` when title already starts with an emoji |
| `toGrams`   | Converts volumetric measurements to rounded grams using `weights_file` (`toGrams .Amount .Name`) |
| `linkRecipe` | Cross-links an ingredient/recipe name to `[Name](relative/path.md)` (excluding self-links) |
| `scale`     | Scales a recipe, ingredient list, amount, number, or fraction/range string (`scale 0.5 .Ingredients`) |
| `lower`     | Converts a string to lowercase                                                |
| `replace`   | Replaces substrings (`replace old new str`)                                   |
| `round`     | Rounds a numeric value to specified decimal places                            |
| `add1`      | Adds `1` to an integer (useful for 1-based step numbering)                    |
| `wrap`      | Word-wraps text to a specified line width                                     |
| `indent`    | Indents lines by a specified number of spaces                                 |

See [`src/document.rs`][6] for how each built-in template and helper function is defined.

## Metadata & YAML Frontmatter

Both legacy `>> key: value` directives and Cooklang YAML frontmatter (`--- ... ---`) [metadata][10] are parsed into:

- `.Metadata`: flat string map (backwards-compatible; rendered by `cook.metadataSection`)
- `.Frontmatter`: structured typed map preserving lists, booleans, numbers, and nested maps for iteration and `toYaml`

```yaml title="Example .cook YAML frontmatter"
---
title: ":bread: Sourdough Loaf"
source: https://example.com/sourdough
serves: 8
total time: 4 hours
tags:
  - bread
  - sourdough
---
```

With a single root `recipe.md.gotmpl`, you can emit Markdown frontmatter and depth-aware asset links across nested category directories:

```go title="cook/recipe.md.gotmpl"
---
comments: true
{{- if .Info.Tags }}
tags:
{{ toYaml .Info.Tags | indent 2 }}
{{- end }}
{{- if .Info.ImageSlug }}
hero: assets/images/{{ .Info.ImageSlug }}
{{- end }}
---
{{ template "cook.headerSection" . }}

{{- if .Info.ImageSlug }}

![{{ .Info.RecipeName }}]({{ .Info.RelRoot }}/assets/images/{{ .Info.ImageSlug }}){ loading=lazy }
{{- end }}

{{ template "cook.tableSection" . }}

{{ template "cook.ingredientsSection" . }}

{{ template "cook.cookwareSection" . }}

{{ template "cook.stepsSection" . -}}

{{ template "cook.sourceSection" . }}
```

## Images

`cook.lazyImageSection` uses [`loading="lazy"`][11], which is [available][12] on modern browsers. To enable Markdown attribute syntax in Zensical / MkDocs, enable [`attr_list`][13]:

```toml title="zensical.toml"
[project.markdown_extensions.attr_list]
```

```yml title="mkdocs.yml"
markdown_extensions:
  - attr_list
```

See [mkdocs-material][14] for reference.

Companion images matching `<Recipe Name>.<ext>` are detected automatically in priority order `.webp`, `.jpg`, `.jpeg`, `.png`. Output Markdown and image slugs are lowercased with spaces replaced by hyphens, e.g. `My Recipe Name.cook -> my-recipe-name.md` and `My Recipe Name.png -> my-recipe-name.png` (`.Info.ImageSlug`).

Pass `--assets-dir <DIR>` to copy companion images (slug-named, keeping their original source extension) during rendering, and optionally `--image-ext webp` to override the extension in `.Info.ImageSlug` when an external tool converts images after rendering.

## Emojis, Weight Conversion & Cross-Linking

When configured, the built-in `cook.ingredients` and `cook.cookware` templates automatically apply emoji prefixes (`--emoji-file` / inline `emoji`), gram annotations (`--weights-file`), and internal recipe cross-links (`--link-dirs`). Custom templates can invoke each helper directly:

```go title="Custom ingredients list using enrichment helpers"
{{- range .Steps }}{{- range .Ingredients }}
- {{ with ingredientEmoji .Name }}{{ . }} {{ end }}{{ if .Amount.Quantity }}{{ round .Amount.Quantity 2 }}{{ if .Amount.Unit }} {{ .Amount.Unit }}{{ end }}{{ with toGrams .Amount .Name }} ({{ . }} g){{ end }}{{ else }}some{{ end }} {{ linkRecipe .Name }}
{{- end }}{{- end }}
```

- **`ingredientEmoji` / `cookwareEmoji` / `emoji`**: Resolves names (case-insensitively, handling prep words and singular/plural variants) against `emoji_file` and inline `emoji` mappings. Returns `""` when unmapped.
- **`titleEmoji`**: Resolves a header emoji for `cook.headerSection` (`titleEmoji .`, `titleEmoji .Info.RecipeName`, or `titleEmoji .Info.RecipeName .Info.Categories`). Returns `""` when `.Metadata.title` (or the input title) already begins with a `:shortcode:` or Unicode emoji; otherwise matches title phrases/words against `titles` then `ingredients`, falling back to `categories` via `.Info.Categories`.
- **`toGrams`**: Uses `cooklang::Converter` to normalize volumetric units (`tsp`, `tbsp`, `cup`, `fl oz`, `pt`, `qt`, `gal`, `ml`, `l`) against `weights_file`. Supports `toGrams .Amount .Name`, `toGrams .Amount.Quantity .Amount.Unit .Name`, or `toGrams .`. Returns `""` for already weight-based units (`g`, `kg`, `oz`, `lb`), unknown units, excluded items, or unmapped ingredients.
- **`linkRecipe`**: Looks up discovered recipes under `--recipe-search-root` (filtered by `--link-dirs` when set) and emits `[Name](relative/path.md)` using forward-slash relative paths between output Markdown files. Excludes self-links and falls back to plain `Name` when unmatched.

See [Configuration](./configuration.md) for the `emoji_file` and `weights_file` schemas.

## Aggregated Ingredients & Scaling (`scale`)

Repeated ingredients and cookware across multiple steps are aggregated into top-level `.Ingredients` and `.Cookware` using `cooklang-rs` unit grouping (e.g., `3 cups flour` in Step 1 + `1/2 cup flour` in Step 2 -> `3 1/2 cups flour`), while per-step `.Steps[*].Ingredients` and `.Steps[*].Cookware` remain intact for backwards compatibility.

Use the `scale` helper to render multi-serving tabs (fractions, mixed numbers, ranges, and servings metadata are scaled automatically):

```go title="Multi-batch ingredient tabs"
## :salt: Ingredients

=== "1x Batch ({{ .Metadata.serves }})"
{{- range .Ingredients }}
    - {{ if .Amount.QuantityRaw }}{{ .Amount.QuantityRaw }}{{ if .Amount.Unit }} {{ .Amount.Unit }}{{ end }}{{ else }}some{{ end }} {{ .Name }}
{{- end }}

=== "1/2 Batch ({{ (scale 0.5 .).Metadata.serves }})"
{{- range scale 0.5 .Ingredients }}
    - {{ if .Amount.QuantityRaw }}{{ .Amount.QuantityRaw }}{{ if .Amount.Unit }} {{ .Amount.Unit }}{{ end }}{{ else }}some{{ end }} {{ .Name }}
{{- end }}

=== "2x Batch ({{ (scale 2 .).Metadata.serves }})"
{{- range scale 2 .Ingredients }}
    - {{ if .Amount.QuantityRaw }}{{ .Amount.QuantityRaw }}{{ if .Amount.Unit }} {{ .Amount.Unit }}{{ end }}{{ else }}some{{ end }} {{ .Name }}
{{- end }}
```

`scale` accepts a numeric or fraction-string factor (`0.5`, `2`, `"1/2"`) and scales an ingredient list (`.Ingredients`), an entire recipe (`.`), a single ingredient/amount, a number, or a fraction/range string (`"1 1/2"`, `"1-2"`).

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

`cook-docs` uses the Rust [`cooklang`][1] parser crate ([`cooklang-rs`][2]) to parse Cooklang recipes and merges them with `Info` and `Config` metadata defined in [`src/types.rs`][3]. The resulting context is passed directly into `gtmpl-ng` templates.

Run `cook-docs --jsonify` to inspect the full parsed data structure on stdout:

```json title="Example parsed cook-docs output"
{
  "Steps": [
    {
      "Directions": "Make 6 pizza balls using tipo zero flour, water, salt and fresh yeast. Put in a fridge for 2 days.",
      "Timers": [
        {
          "Name": "",
          "Duration": 2.0,
          "Unit": "days"
        }
      ],
      "Ingredients": [
        {
          "Name": "tipo zero flour",
          "Amount": {
            "IsNumeric": true,
            "Quantity": 820.0,
            "QuantityRaw": "820",
            "Unit": "g"
          }
        },
        {
          "Name": "water",
          "Amount": {
            "IsNumeric": true,
            "Quantity": 533.0,
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
          "Quantity": 1.0,
          "QuantityRaw": ""
        }
      ],
      "Comments": []
    }
  ],
  "Ingredients": [
    {
      "Name": "tipo zero flour",
      "Amount": {
        "IsNumeric": true,
        "Quantity": 820.0,
        "QuantityRaw": "820",
        "Unit": "g"
      }
    }
  ],
  "Cookware": [
    {
      "IsNumeric": false,
      "Name": "fridge",
      "Quantity": 1.0,
      "QuantityRaw": ""
    }
  ],
  "Metadata": {
    "servings": "6",
    "source": "https://www.somewebsite.com/pizza-balls"
  },
  "Frontmatter": {
    "servings": "6",
    "source": "https://www.somewebsite.com/pizza-balls"
  },
  "Config": {
    "DryRun": false,
    "Jsonify": true,
    "IgnoreFile": ".cookdocsignore",
    "RecipeSearchRoot": "testdata",
    "LogLevel": "info",
    "TemplateFiles": [
      "recipe.md.gotmpl"
    ],
    "WordWrap": 120,
    "OutputDir": "",
    "AssetsDir": "",
    "ImageExt": ""
  },
  "Info": {
    "ImageFileName": "My Test Recipe.png",
    "ImageFilePath": "/path/to/cook-docs/testdata/My Test Recipe.png",
    "ImageSlug": "my-test-recipe.png",
    "NewRecipeFilePath": "/path/to/cook-docs/testdata/my-test-recipe.md",
    "RecipeName": "My Test Recipe",
    "RecipeSlug": "my-test-recipe",
    "RecipeFilePath": "/path/to/cook-docs/testdata/My Test Recipe.cook",
    "RelRoot": ".",
    "Categories": [],
    "DefaultTags": [],
    "Tags": []
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

[1]: <https://crates.io/crates/cooklang>
[2]: <https://github.com/cooklang/cooklang-rs>
[3]: <https://github.com/nicholaswilde/cook-docs/blob/main/src/types.rs>
[4]: <https://pkg.go.dev/text/template#hdr-Text_and_spaces>
[5]: <https://crates.io/crates/gtmpl-ng>
[6]: <https://github.com/nicholaswilde/cook-docs/blob/main/src/document.rs>
[8]: <https://github.github.com/gfm/#block-quotes>
[9]: <https://squidfunk.github.io/mkdocs-material/reference/admonitions/>
[10]: <https://cooklang.org/docs/spec/#metadata>
[11]: <https://developer.mozilla.org/en-US/docs/Web/API/HTMLImageElement/loading>
[12]: <https://caniuse.com/loading-lazy-attr>
[13]: <https://python-markdown.github.io/extensions/attr_list/>
[14]: <https://squidfunk.github.io/mkdocs-material/reference/images/#image-lazy-loading>

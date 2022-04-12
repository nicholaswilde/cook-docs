package document

import (
  "fmt"
  "encoding/json"
  "text/template"
  "os"

  "github.com/Masterminds/sprig/v3"
  "github.com/aquilax/cooklang-go"
	log "github.com/sirupsen/logrus"
)

func PrintDocumentation(recipePath string, r *cooklang.Recipe) {
  log.Infof("Generating markdown file for recipe %s", recipePath)
  j, err := json.MarshalIndent(r, "", "  ")
  if err != nil {
    log.Fatal(err)
  }
  fmt.Println(string(j))

  const recipe = `
# Test Recipe

![](../assets/images/crispy-chicken-less-sliders.png)

| :fork_and_knife_with_plate: Serves | :timer_clock: Total Time |
|:------:|:----------:|
| {{.Metadata.servings}}      | 25 minutes |

## :salt: Ingredients
{{ range .Steps }}
{{- range .Ingredients }}
- {{.Amount.Quantity}} {{.Amount.Unit}} {{.Name}}
{{- end }}
{{- end }}

## :cooking: Cookware
{{ range .Steps }}
{{- if .Cookware }}
{{- range .Cookware }}
- {{.Name}}
{{- end }}
{{- end }}
{{- end }}

## :pencil: Instructions

{{- range $i, $a := .Steps }}

### Step {{inc $i}}

{{ .Directions }}

{{- end }}

## Source
- {{.Metadata.source}}
`
  // https://stackoverflow.com/a/25690905
  funcMap := template.FuncMap{
        // The name "inc" is what the function will be called in the template text.
        "inc": func(i int) int {
            return i + 1
        },
    }
  t := template.Must(template.New("recipe").Funcs(funcMap).Funcs(sprig.TxtFuncMap()).Parse(recipe))
  err = t.Execute(os.Stdout, r)
  if err != nil {
    log.Println("executing template:", err)
  }
}

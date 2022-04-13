package document

import (
  "fmt"
  "encoding/json"
  //"text/template"
  "os"

  //"github.com/Masterminds/sprig/v3"
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
{{- define "custom.helm.url" -}}
https://k8s-at-home.com/charts/
{{- end -}}
# Test Recipe

![](../assets/images/crispy-chicken-less-sliders.png)

| :fork_and_knife_with_plate: Serves | :timer_clock: Total Time |
|:------:|:----------:|
| {{.Metadata.servings}} | 25 minutes |

## :salt: Ingredients
{{ range .Steps }}
{{- range .Ingredients }}
- {{.Amount.Quantity}} {{.Amount.Unit}} {{.Name}}
{{- end }}
{{- end }}

{{ range .Steps }}
{{- if .Cookware }}
{{- range .Cookware }}
- {{.Name}}
{{- end }}
{{- end }}
{{- end }}

## :pencil: Instructions

{{- range $i, $a := .Steps }}

### Step {{add1 $i}}

{{ .Directions }}

{{- end }}

{{ if .Metadata.source -}}
## Source
- {{.Metadata.source}}
{{- end}}
`
  t := newRecipeDocumentationTemplate(recipe, recipePath)
  err = t.Execute(os.Stdout, r)
  if err != nil {
    log.Println("executing template:", err)
  }
}

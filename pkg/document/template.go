package document

import(
  "os"
  "errors"
  "strings"
  "text/template"
  "path/filepath"
  "io/ioutil"
  "fmt"

  "github.com/aquilax/cooklang-go"
  "github.com/Masterminds/sprig/v3"
  "github.com/nicholaswilde/cook-docs/pkg/cook"
  "github.com/nicholaswilde/cook-docs/pkg/util"
  log "github.com/sirupsen/logrus"
)

const defaultDocumentationTemplate =`{{ template "recipe.headerSection" . }}

{{ template "recipe.imageSection" . }}

{{ template "recipe.tableSection" . }}

{{ template "recipe.ingredientsSection" . }}

{{ template "recipe.cookwareSection" . }}

{{ template "recipe.stepsSection" . }}

{{ template "recipe.sourceSection" . }}
`

func getHeaderTemplate() string {
  templateBuilder := strings.Builder{}

  templateBuilder.WriteString(`{{ define "recipe.headerSection" }}`)
  templateBuilder.WriteString("# {{ .Metadata.title }}")
  templateBuilder.WriteString("{{ end }}")

  return templateBuilder.String()
}

func getImageTemplate() string {
  templateBuilder := strings.Builder{}

  templateBuilder.WriteString(`{{ define "recipe.imageSection" }}`)
  templateBuilder.WriteString("{{ if .Metadata.ImageName }}")
  templateBuilder.WriteString(`![](../assets/images/{{ lower .Metadata.ImageName | replace " " "-" }})`)
  templateBuilder.WriteString("{{ end }}")
  templateBuilder.WriteString("{{ end }}")

  return templateBuilder.String()
}

func getTableTemplate() string {
  templateBuilder := strings.Builder{}

  templateBuilder.WriteString(`{{ define "recipe.tableSection" }}`)
  templateBuilder.WriteString("{{ if or .Metadata.servings .Metadata.serves }}")
  templateBuilder.WriteString("| :fork_and_knife_with_plate: Serves | :timer_clock: Total Time |\n")
  templateBuilder.WriteString("|:----------------------------------:|:-----------------------: |\n")
  templateBuilder.WriteString("| {{ if .Metadata.servings }}{{ .Metadata.servings }}{{ else if .Metadata.serves }}{{ .Metadata.serves }}{{ end }} | {{ sumTimers .Steps }} |")
  templateBuilder.WriteString("{{ else }}")
  templateBuilder.WriteString("| :timer_clock: Total Time |\n")
  templateBuilder.WriteString("|:-----------------------: |\n")
  templateBuilder.WriteString("| {{ sumTimers .Steps }} |")
  templateBuilder.WriteString("{{ end }}")
  templateBuilder.WriteString("{{ end }}")

  return templateBuilder.String()
}

func getIngredientsTemplate() string {
  templateBuilder := strings.Builder{}

  templateBuilder.WriteString(`{{ define "recipe.ingredientsHeader" }}`)
  templateBuilder.WriteString("## :salt: Ingredients")
  templateBuilder.WriteString("{{ end }}")

  templateBuilder.WriteString(`{{ define "recipe.ingredients" }}`)
  templateBuilder.WriteString("{{ range .Steps }}{{- range .Ingredients }}\n- {{.Amount.Quantity}} {{.Amount.Unit}} {{.Name}}{{- end }}{{- end }}")
  templateBuilder.WriteString("{{ end }}")

  templateBuilder.WriteString(`{{ define "recipe.ingredientsSection" }}`)
  templateBuilder.WriteString(`{{ template "recipe.ingredientsHeader" . }}`)
  templateBuilder.WriteString("\n")
  templateBuilder.WriteString(`{{ template "recipe.ingredients" . }}`)
  templateBuilder.WriteString("{{ end }}")

  return templateBuilder.String()
}

func getCookwareTemplate() string {
  templateBuilder := strings.Builder{}

  templateBuilder.WriteString(`{{ define "recipe.cookwareHeader" }}`)
  templateBuilder.WriteString("## :cooking: Cookware")
  templateBuilder.WriteString("{{ end }}")

  templateBuilder.WriteString(`{{ define "recipe.cookware" }}`)
  templateBuilder.WriteString("{{ range .Steps }}{{- range .Cookware }}\n- {{.Name}}{{- end }}{{- end }}")
  templateBuilder.WriteString("{{ end }}")

  templateBuilder.WriteString(`{{ define "recipe.cookwareSection" }}`)
  templateBuilder.WriteString(`{{ template "recipe.cookwareHeader" . }}`)
  templateBuilder.WriteString("\n")
  templateBuilder.WriteString(`{{ template "recipe.cookware" . }}`)
  templateBuilder.WriteString("{{ end }}")

  return templateBuilder.String()
}

func getStepsTemplate() string {
  templateBuilder := strings.Builder{}

  templateBuilder.WriteString(`{{ define "recipe.stepsHeader" }}`)
  templateBuilder.WriteString("## :pencil: Instructions")
  templateBuilder.WriteString("{{ end }}")

  templateBuilder.WriteString(`{{ define "recipe.steps" }}`)
  templateBuilder.WriteString("{{ range $i, $a := .Steps }}\n\n### Step {{add1 $i}}\n\n{{ .Directions }}{{- end }}")
  templateBuilder.WriteString("{{ end }}")

  templateBuilder.WriteString(`{{ define "recipe.stepsSection" }}`)
  templateBuilder.WriteString(`{{ template "recipe.stepsHeader" . }}`)
  templateBuilder.WriteString(`{{ template "recipe.steps" . }}`)
  templateBuilder.WriteString("{{ end }}")

  return templateBuilder.String()
}

func getSourceTemplate() string {
  templateBuilder := strings.Builder{}

  templateBuilder.WriteString(`{{ define "recipe.sourceHeader" }}`)
  templateBuilder.WriteString("## :link: Source")
  templateBuilder.WriteString("{{ end }}")

  templateBuilder.WriteString(`{{ define "recipe.source" }}`)
  templateBuilder.WriteString("- {{ .Metadata.source }}")
  templateBuilder.WriteString("{{ end }}")

  templateBuilder.WriteString(`{{ define "recipe.sourceSection" }}`)
  templateBuilder.WriteString("{{ if .Metadata.source }}")
  templateBuilder.WriteString(`{{ template "recipe.sourceHeader" . }}`)
  templateBuilder.WriteString("\n")
  templateBuilder.WriteString(`{{ template "recipe.source" . }}`)
  templateBuilder.WriteString("{{ end }}")
  templateBuilder.WriteString("{{ end }}")

  return templateBuilder.String()
}

func getMetadataTemplate() string {
  templateBuilder := strings.Builder{}

  templateBuilder.WriteString(`{{ define "recipe.metadataHeader" }}`)
  templateBuilder.WriteString("## Metadata")
  templateBuilder.WriteString("{{ end }}")

  templateBuilder.WriteString(`{{ define "recipe.metadata" }}`)
  templateBuilder.WriteString(`{{ range $key, $value := .Metadata }}\n- {{ $key }}: {{ $value }}{{ end }}`)
  templateBuilder.WriteString("{{ end }}")

  templateBuilder.WriteString(`{{ define "recipe.metadataSection" }}`)
  templateBuilder.WriteString(`{{ template "recipe.metadataHeader" . }}`)
  templateBuilder.WriteString("\n")
  templateBuilder.WriteString(`{{ template "recipe.metadata" . }}`)
  templateBuilder.WriteString("{{ end }}")

  return templateBuilder.String()
}

func getCommentsTemplate() string {
  templateBuilder := strings.Builder{}

  templateBuilder.WriteString(`{{ define "recipe.commentsHeader" }}`)
  templateBuilder.WriteString("## Comments")
  templateBuilder.WriteString("{{ end }}")

  templateBuilder.WriteString(`{{ define "recipe.comments" }}`)
  //templateBuilder.WriteString("{{ range .Steps }}{{- range .Comments }}\n- {{.}}{{- end }}{{- end }}")
  templateBuilder.WriteString("{{ end }}")

  templateBuilder.WriteString(`{{ define "recipe.commentsSection" }}`)
  templateBuilder.WriteString("{{ range .Steps }}")
  templateBuilder.WriteString("{{ if .Comments }}")
  templateBuilder.WriteString(`{{ template "recipe.commentsHeader" . }}`)
  templateBuilder.WriteString("\n")
  templateBuilder.WriteString(`{{ template "recipe.comments" . }}`)
  templateBuilder.WriteString("{{ end }}")
  templateBuilder.WriteString("{{ end }}")
  templateBuilder.WriteString("{{ end }}")

  return templateBuilder.String()
}

func getDocumentationTemplate(recipeSearchRoot string, recipePath string, templateFiles []string) (string, error){
  templateFilesForRecipe := make([]string, 0)

  var templateNotFound bool

  path := filepath.Dir(recipePath)

  for _, templateFile := range templateFiles {
    var fullTemplatePath string

    if util.IsRelativePath(templateFile) {
      fullTemplatePath = filepath.Join(recipeSearchRoot, templateFile)
    } else if util.IsBaseFilename(templateFile) {
      fullTemplatePath = filepath.Join(path, templateFile)
    }
    _, err := os.Stat(fullTemplatePath);
    if errors.Is(err, os.ErrNotExist) {
      log.Debugf("Did not find template file %s for recipe %s, using default template", fullTemplatePath, recipePath)

      templateNotFound = true
      continue
    }

    templateFilesForRecipe = append(templateFilesForRecipe, fullTemplatePath)
  }

  log.Debugf("Using template files %s for chart %s", templateFiles, recipePath)
  allTemplateContents := make([]byte, 0)

  for _, templateFileForRecipe := range templateFilesForRecipe {
    templateContents, err := ioutil.ReadFile(templateFileForRecipe)
    if err != nil {
      return "", err
    }
    allTemplateContents = append(allTemplateContents, templateContents...)
  }

  if templateNotFound {
    allTemplateContents = append(allTemplateContents, []byte(defaultDocumentationTemplate)...)
  }

  return string(allTemplateContents), nil
}

func getDocumentationTemplates(recipeSearchRoot string, recipePath string, templateFiles []string) ([]string, error) {
  documentationTemplate, err := getDocumentationTemplate(recipeSearchRoot, recipePath, templateFiles)

  if err != nil {
		log.Errorf("Failed to read documentation template for recipe %s: %s", recipePath, err)
		return nil, err
	}

  return []string{
    getHeaderTemplate(),
    getImageTemplate(),
    getTableTemplate(),
    getIngredientsTemplate(),
    getCookwareTemplate(),
    getStepsTemplate(),
    getSourceTemplate(),
    getMetadataTemplate(),
    getCommentsTemplate(),
    documentationTemplate,
  }, nil
}

func sum2(steps []cooklang.Step) float64 {
    var sum float64
    var v float64
    for _, s := range steps {
      for _, t := range s.Timers {
        v = t.Duration
        sum += v
      }
    }
    return sum
}

func newRecipeDocumentationTemplate(recipeSearchRoot string, recipeInfo cook.RecipeDocumentationInfo, templateFiles []string) (*template.Template, error) {
  documentationTemplate := template.New(recipeInfo.RecipePath)
  documentationTemplate.Funcs(sprig.TxtFuncMap())
  documentationTemplate.Funcs(template.FuncMap{"sumTimers": func(steps []cooklang.Step) string {
    var sum float64
    for _, s := range steps {
      for _, t := range s.Timers {
        switch t.Unit {
        case "day","days":
          sum = sum + t.Duration*60*24
        case "hour","hours":
          sum = sum + t.Duration*60
        case "minute","minutes":
          sum += t.Duration
        }
      }
    }
    if (sum>1440){
      sum = sum/1440
      return fmt.Sprintf("%.2f days", sum)
    } else if (sum>60) {
      sum = sum/60
      return fmt.Sprintf("%.2f hours", sum)
    } else {
      return fmt.Sprintf("%f minutes", sum)
    }
  }})

  goTemplateList, err := getDocumentationTemplates(recipeSearchRoot, recipeInfo.RecipePath, templateFiles)
  if err != nil {
		return nil, err
	}
  for _, t := range goTemplateList {
		_, err := documentationTemplate.Parse(t)

		if err != nil {
			return nil, err
		}
	}

  return documentationTemplate, nil
}

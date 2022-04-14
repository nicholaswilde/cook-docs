package document

import(
  "os"
  "errors"
  "strings"
  "text/template"
  "path/filepath"
  "io/ioutil"

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
	headerTemplateBuilder := strings.Builder{}

	headerTemplateBuilder.WriteString(`{{ define "recipe.headerSection" }}`)
	headerTemplateBuilder.WriteString("# Test Recipe")
	headerTemplateBuilder.WriteString("{{ end }}")

	return headerTemplateBuilder.String()
}

func getImageTemplate() string {
  imageTemplateBuilder := strings.Builder{}

	imageTemplateBuilder.WriteString(`{{ define "recipe.imageSection" }}`)
	imageTemplateBuilder.WriteString("![](../assets/images/crispy-chicken-less-sliders.png)")
	imageTemplateBuilder.WriteString("{{ end }}")

  return imageTemplateBuilder.String()
}

func getTableTemplate() string {
  tableTemplateBuilder := strings.Builder{}

  tableTemplateBuilder.WriteString(`{{ define "recipe.tableSection" }}`)
  tableTemplateBuilder.WriteString("| :fork_and_knife_with_plate: Serves | :timer_clock: Total Time |\n")
  tableTemplateBuilder.WriteString("|:----------------------------------:|:-----------------------: |\n")
  tableTemplateBuilder.WriteString("| {{.Metadata.servings}} | 25 minutes |")
  tableTemplateBuilder.WriteString("{{ end }}")

  return tableTemplateBuilder.String()
}

func getIngredientsTemplate() string {
  ingredientsTemplateBuilder := strings.Builder{}

  ingredientsTemplateBuilder.WriteString(`{{ define "recipe.ingredientsSection" }}`)
  ingredientsTemplateBuilder.WriteString("## :salt: Ingredients\n")
  ingredientsTemplateBuilder.WriteString("{{ range .Steps }}{{- range .Ingredients }}\n- {{.Amount.Quantity}} {{.Amount.Unit}} {{.Name}}{{- end }}{{- end }}")
  ingredientsTemplateBuilder.WriteString("{{ end }}")

  return ingredientsTemplateBuilder.String()
}

func getCookwareTemplate() string {
  cookwareTemplateBuilder := strings.Builder{}

	cookwareTemplateBuilder.WriteString(`{{ define "recipe.cookwareSection" }}`)
	cookwareTemplateBuilder.WriteString("## Cookware\n")
  cookwareTemplateBuilder.WriteString("{{ range .Steps }}{{- range .Cookware }}\n- {{.Name}}{{- end }}{{- end }}")
	cookwareTemplateBuilder.WriteString("{{ end }}")

  return cookwareTemplateBuilder.String()
}

func getStepsTemplate() string {
  stepsTemplateBuilder := strings.Builder{}

	stepsTemplateBuilder.WriteString(`{{ define "recipe.stepsSection" }}`)
	stepsTemplateBuilder.WriteString("## :pencil: Instructions")
  stepsTemplateBuilder.WriteString("{{ range $i, $a := .Steps }}\n\n### Step {{add1 $i}}\n\n{{ .Directions }}{{ end }}")
	stepsTemplateBuilder.WriteString("{{ end }}")

  return stepsTemplateBuilder.String()
}

func getSourceTemplate() string {
  sourceTemplateBuilder := strings.Builder{}

	sourceTemplateBuilder.WriteString(`{{ define "recipe.sourceSection" }}`)
  sourceTemplateBuilder.WriteString("{{ if .Metadata.source }}")
	sourceTemplateBuilder.WriteString("## :link: Source\n")
  sourceTemplateBuilder.WriteString("- {{ .Metadata.source }}")
	sourceTemplateBuilder.WriteString("{{ end }}")
	sourceTemplateBuilder.WriteString("{{ end }}")

  return sourceTemplateBuilder.String()
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
    documentationTemplate,
  }, nil
}

func newRecipeDocumentationTemplate(recipeSearchRoot string, recipeInfo cook.RecipeDocumentationInfo, templateFiles []string) (*template.Template, error) {
  documentationTemplate := template.New(recipeInfo.RecipePath)
  documentationTemplate.Funcs(sprig.TxtFuncMap())
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

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

const defaultDocumentationTemplate =`{{ template "recipe.header" . }}
`

func getHeaderTemplate() string {
	headerTemplateBuilder := strings.Builder{}
	headerTemplateBuilder.WriteString(`{{ define "recipe.header" }}`)
	headerTemplateBuilder.WriteString("# Test Recipe\n")
	headerTemplateBuilder.WriteString("{{ end }}")

	return headerTemplateBuilder.String()
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

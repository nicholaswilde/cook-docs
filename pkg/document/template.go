package document

import(
  "os"
  "text/template"
  "io/ioutil"
  "github.com/Masterminds/sprig/v3"
  "github.com/nicholaswilde/cook-docs/pkg/cook"
  "github.com/nicholaswilde/cook-docs/pkg/util"
  log "github.com/sirupsen/logrus"
)

const defaultDocumentationTemplate =`
# Test Recipe
`

func getDocumentationTemplate(recipePath string, templateFiles []string) (string, error){
  templateFilesForRecipe := make([]string, 0)

  var templateNotFound bool

  for _, templateFile := range templateFiles {
    var fullTemplatePath string
    if util.IsRelativePath(templateFile) {
    //  fullTemplatePath = path.Join(chartSearchRoot, templateFile)
    //} else if util.IsBaseFilename(templateFile) {
    //  fullTemplatePath = path.Join(chartDirectory, templateFile)
    }
    _, err := os.Stat(fullTemplatePath);
    if os.IsNotExist(err) {
      log.Debugf("Did not find template file %s for recipe %s, using default template", templateFile, recipePath)

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

func getDocumentationTemplates(recipePath string, templateFiles []string) ([]string, error) {
  _, err := getDocumentationTemplate(recipePath, templateFiles)

  if err != nil {
		log.Errorf("Failed to read documentation template for recipe %s: %s", recipePath, err)
		return nil, err
	}

  return []string{
  }, nil
}

func newRecipeDocumentationTemplate(recipe string, recipeInfo cook.RecipeDocumentationInfo, templateFiles []string) (*template.Template, error) {
  t := template.New(recipeInfo.RecipePath)
  t.Funcs(sprig.TxtFuncMap())
  _, err := getDocumentationTemplates(recipeInfo.RecipePath, templateFiles)
  if err != nil {
		return nil, err
	}
  t.Parse(recipe)
  return t, nil
}

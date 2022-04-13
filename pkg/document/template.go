package document

import(
  //"os"
  "text/template"
  "github.com/Masterminds/sprig/v3"
  "github.com/nicholaswilde/cook-docs/pkg/cook"
  //log "github.com/sirupsen/logrus"
)

func getDocumentationTemplates(recipePath string, templateFiles []string) ([]string, error) {
  return nil, nil
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

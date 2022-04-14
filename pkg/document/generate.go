package document

import (
  "encoding/json"
  "os"

  "github.com/aquilax/cooklang-go"
  "github.com/nicholaswilde/cook-docs/pkg/cook"
	log "github.com/sirupsen/logrus"
)

func PrintDocumentation(recipeSearchRoot string, recipeData *cooklang.Recipe, recipeInfo cook.RecipeDocumentationInfo, templateFiles []string) {
  log.Infof("Generating markdown file for recipe %s", recipeInfo.RecipePath)
  j, err := json.MarshalIndent(recipeData, "", "  ")
  if err != nil {
    log.Fatal(err)
  }
  log.Debug(string(j))

  t, err := newRecipeDocumentationTemplate(recipeSearchRoot, recipeInfo, templateFiles)
  if err != nil {
    log.Println("executing template:", err)
  }
  err = t.Execute(os.Stdout, recipeData)
  if err != nil {
    log.Println("executing template:", err)
  }
}

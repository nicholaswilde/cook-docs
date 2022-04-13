package cook

import (
  "fmt"
  "strings"
  "path/filepath"
)

type RecipeDocumentationInfo struct {
  RecipePath string
  RecipeName string
}

func ParseRecipeInformation(recipePath string) (RecipeDocumentationInfo, error) {
  var recipeDocInfo RecipeDocumentationInfo
  var err error
  recipeDocInfo.RecipePath = recipePath
  fileName := filepath.Base(recipePath)
  recipeDocInfo.RecipeName = strings.TrimSuffix(fileName, filepath.Ext(fileName))
  fmt.Println(recipeDocInfo.RecipeName)
  return recipeDocInfo, err
}

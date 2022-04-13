package cook

import (
  "os"
  "errors"
  "strings"
  "path/filepath"
)

type RecipeDocumentationInfo struct {
  ImagePath string
  RecipePath string
  RecipeName string
}

func ParseRecipeInformation(recipePath string) (RecipeDocumentationInfo, error) {
  var recipeDocInfo RecipeDocumentationInfo
  var err error
  recipeDocInfo.RecipePath = recipePath
  fileName := filepath.Base(recipePath)
  recipeDocInfo.RecipeName = strings.TrimSuffix(fileName, filepath.Ext(fileName))
  path := filepath.Dir(recipePath)
  imagePath := filepath.Join(path, recipeDocInfo.RecipeName) + ".jpg"
  _, err = os.Stat(imagePath)
  if errors.Is(err, os.ErrNotExist) {
    imagePath = filepath.Join(path, recipeDocInfo.RecipeName) + ".png"
  }
  _, err = os.Stat(imagePath)
  if !errors.Is(err, os.ErrNotExist) {
    recipeDocInfo.ImagePath = imagePath
  }
  return recipeDocInfo, err
}

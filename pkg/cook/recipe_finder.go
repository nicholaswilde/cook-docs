package cook

import (
  "os"
  "path/filepath"
)

func FindRecipeDirectories(recipeSearchRoot string) ([]string, error) {
  recipeDirs := make([]string, 0)
  err := filepath.Walk(recipeSearchRoot, func(path string, info os.FileInfo, err error) error {
    if err != nil {
      return err
    }
    if filepath.Ext(path) == ".cook" {
      recipeDirs = append(recipeDirs, path)
    }
    return nil
  })

  return recipeDirs, err
}

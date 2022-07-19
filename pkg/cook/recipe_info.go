package cook

import (
	"errors"
	"os"
	"path/filepath"
	"strings"

	"github.com/nicholaswilde/cook-docs/pkg/types"
	"github.com/aquilax/cooklang-go"
	log "github.com/sirupsen/logrus"
)

type RecipeDocumentationInfo struct {
	ImagePath   string
	RecipeFilePath  string
	RecipeName  string
	NewRecipeFilePath string
}

func GetNewRecipeFilePath(info types.Info) string {
	path := filepath.Dir(info.RecipeFilePath)
	fileName := strings.Replace(info.RecipeName, " ", "-", -1)
	fileName = strings.ToLower(fileName) + ".md"
	return filepath.Join(path, fileName)
}

func GetRecipeName(recipePath string) string {
	fileName := filepath.Base(recipePath)
	return strings.TrimSuffix(fileName, filepath.Ext(fileName))
}

func GetImagePath(info types.Info) (string, error) {
	var err error
	path := filepath.Dir(info.RecipeFilePath)
	imagePath := filepath.Join(path, info.RecipeName) + ".jpg"
	_, err = os.Stat(imagePath)
	if errors.Is(err, os.ErrNotExist) {
		imagePath = filepath.Join(path, info.RecipeName) + ".png"
	}
	_, err = os.Stat(imagePath)
	if errors.Is(err, os.ErrNotExist) {
		log.Warnf("Image file %s missing.", imagePath)
		return "", err
	}
	return imagePath, nil
}

func ParseFile(recipePath string, config *types.Config) (*types.Recipe, error) {
	var info types.Info
	var recipe types.Recipe

	info.RecipeFilePath = recipePath

	info.RecipeName = GetRecipeName(recipePath)

	info.NewRecipeFilePath = GetNewRecipeFilePath(info)

	imagePath, err := GetImagePath(info)

	if err == nil {
		info.ImageFilePath = imagePath
		info.ImageFileName = filepath.Base(imagePath)
	}

	recipeData, err := cooklang.ParseFile(recipePath)

	if err != nil {
		log.Warnf("Error parsing file for recipe %s, skipping: %s", recipePath, err)
		return nil, err
	}

	recipe.Metadata = recipeData.Metadata
	recipe.Steps = recipeData.Steps
	recipe.Info = info
	recipe.Config = *config

	return &recipe, nil
}

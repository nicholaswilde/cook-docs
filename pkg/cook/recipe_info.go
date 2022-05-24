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
	RecipePath  string
	RecipeName  string
	NewFileName string
}

// TODO: Replace RecipeDocumentationInfo with types.Info

func GetNewFileName(recipeDocInfo RecipeDocumentationInfo) string {
	path := filepath.Dir(recipeDocInfo.RecipePath)
	fileName := strings.Replace(recipeDocInfo.RecipeName, " ", "-", -1)
	fileName = strings.ToLower(fileName) + ".md"
	return filepath.Join(path, fileName)
}

func GetRecipeName(recipePath string) string {
	fileName := filepath.Base(recipePath)
	return strings.TrimSuffix(fileName, filepath.Ext(fileName))
}

func GetImagePath(recipeDocInfo RecipeDocumentationInfo) (string, error) {
	var err error
	path := filepath.Dir(recipeDocInfo.RecipePath)
	imagePath := filepath.Join(path, recipeDocInfo.RecipeName) + ".jpg"
	_, err = os.Stat(imagePath)
	if errors.Is(err, os.ErrNotExist) {
		imagePath = filepath.Join(path, recipeDocInfo.RecipeName) + ".png"
	}
	_, err = os.Stat(imagePath)
	if errors.Is(err, os.ErrNotExist) {
		log.Warnf("Image file %s missing.", imagePath)
		return "", err
	}
	return imagePath, nil
}

func ParseRecipeInformation(recipePath string) RecipeDocumentationInfo {
	var recipeDocInfo RecipeDocumentationInfo

	recipeDocInfo.RecipePath = recipePath

	recipeDocInfo.RecipeName = GetRecipeName(recipePath)

	recipeDocInfo.NewFileName = GetNewFileName(recipeDocInfo)

	imagePath, err := GetImagePath(recipeDocInfo)

	if err == nil {
		recipeDocInfo.ImagePath = imagePath
	}

	return recipeDocInfo
}

func ParseFile(recipePath string, config *types.Config) (types.Recipe, error) {
	var info types.Info
	var recipe types.Recipe

	info.RecipePath = recipePath

	info.RecipeName = GetRecipeName(recipePath)

	// info.NewFileName = GetNewFileName(info)

	// imagePath, err := GetImagePath(info)

	// if err == nil {
		// info.ImagePath = imagePath
	// }

	return recipe, nil
}

func MergeRecipeData(recipeInfo RecipeDocumentationInfo, recipeData *cooklang.Recipe) *cooklang.Recipe {
	var r types.Recipe
	r.Steps = recipeData.Steps
	r.Metadata = recipeData.Metadata

	recipeData.Metadata["title"] = recipeInfo.RecipeName

	recipeData.Metadata["ImageName"] = filepath.Base(recipeInfo.ImagePath)

	return recipeData
}

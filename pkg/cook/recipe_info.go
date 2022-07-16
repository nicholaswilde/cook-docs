package cook

import (
	"errors"
	"os"
	"path/filepath"
	"strings"

	"github.com/aquilax/cooklang-go"
	log "github.com/sirupsen/logrus"
)

type RecipeDocumentationInfo struct {
	ImagePath   string
	RecipePath  string
	RecipeName  string
	NewFileName string
}

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

func MergeRecipeData(recipeInfo RecipeDocumentationInfo, recipeData *cooklang.Recipe) *cooklang.Recipe {

	recipeData.Metadata["title"] = recipeInfo.RecipeName

	if len(recipeInfo.ImagePath) > 0 {
		recipeData.Metadata["ImageName"] = filepath.Base(recipeInfo.ImagePath)
	}

	return recipeData
}

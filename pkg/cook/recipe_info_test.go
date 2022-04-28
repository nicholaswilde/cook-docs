package cook

import (
	"testing"

	"github.com/aquilax/cooklang-go"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGetNewFileName(t *testing.T) {
	var recipeInfo RecipeDocumentationInfo
	recipeInfo.RecipePath = "testdata/Recipe.cook"
	recipeInfo.RecipeName = "Recipe"
	fileName := GetNewFileName(recipeInfo)
	assert.Equal(t, "testdata/recipe.md", fileName)
}

func TestGetRecipeName(t *testing.T) {
	recipeName := GetRecipeName("testdata/Test Recipe.cook")
	assert.Equal(t, "Test Recipe", recipeName)
}

func TestGetImagePath(t *testing.T) {
	var recipeInfo RecipeDocumentationInfo
	recipeInfo.RecipePath = "testdata/Recipe.cook"
	recipeInfo.RecipeName = "Recipe"
	imagePath, err := GetImagePath(recipeInfo)
	require.NoError(t, err)
	assert.Equal(t, "testdata/Recipe.png", imagePath)

	recipeInfo.RecipePath = "testdata/Recipe2.cook"
	recipeInfo.RecipeName = "Recipe2"
	imagePath, err = GetImagePath(recipeInfo)
	require.NoError(t, err)
	assert.Equal(t, "testdata/Recipe2.jpg", imagePath)

	recipeInfo.RecipePath = "testdata/Recipe3.cook"
	recipeInfo.RecipeName = "Recipe3"
	_, err = GetImagePath(recipeInfo)
	require.Error(t, err)
}

func TestMergeRecipeData(t *testing.T) {
	var recipeInfo RecipeDocumentationInfo
	var recipeData = cooklang.Recipe{
		Steps:    make([]cooklang.Step, 0),
		Metadata: make(map[string]string),
	}
	recipeInfo.RecipeName = "Recipe"
	recipeInfo.ImagePath = "testdata/recipe.jpg"
	recipeData2 := MergeRecipeData(recipeInfo, &recipeData)
	assert.Equal(t, recipeData2.Metadata["title"], "Recipe")
	assert.Equal(t, recipeData2.Metadata["ImageName"], "recipe.jpg")
}

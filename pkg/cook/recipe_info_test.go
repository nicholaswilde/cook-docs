package cook

import (
	"testing"

	"github.com/nicholaswilde/cook-docs/pkg/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGetNewRecipeFilePath(t *testing.T) {
	var recipeInfo types.Info
	recipeInfo.RecipeFilePath = "testdata/Recipe.cook"
	recipeInfo.RecipeName = "Recipe"
	fileName := GetNewRecipeFilePath(recipeInfo)
	assert.Equal(t, "testdata/recipe.md", fileName)
}

func TestGetRecipeName(t *testing.T) {
	recipeName := GetRecipeName("testdata/Test Recipe.cook")
	assert.Equal(t, "Test Recipe", recipeName)
}

func TestGetImagePath(t *testing.T) {
	var recipeInfo types.Info
	recipeInfo.RecipeFilePath = "testdata/Recipe.cook"
	recipeInfo.RecipeName = "Recipe"
	imagePath, err := GetImagePath(recipeInfo)
	require.NoError(t, err)
	assert.Equal(t, "testdata/Recipe.png", imagePath)

	recipeInfo.RecipeFilePath = "testdata/Recipe2.cook"
	recipeInfo.RecipeName = "Recipe2"
	imagePath, err = GetImagePath(recipeInfo)
	require.NoError(t, err)
	assert.Equal(t, "testdata/Recipe2.jpg", imagePath)

	recipeInfo.RecipeFilePath = "testdata/Recipe3.cook"
	recipeInfo.RecipeName = "Recipe3"
	_, err = GetImagePath(recipeInfo)
	require.Error(t, err)
}

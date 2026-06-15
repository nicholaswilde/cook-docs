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
	
	// Test default behavior (no OutputDir)
	fileName := GetNewRecipeFilePath(recipeInfo, nil)
	assert.Equal(t, "testdata/recipe.md", fileName)
	
	fileName = GetNewRecipeFilePath(recipeInfo, &types.Config{})
	assert.Equal(t, "testdata/recipe.md", fileName)

	// Test with OutputDir
	config := &types.Config{
		OutputDir:        "/tmp/out",
		RecipeSearchRoot: "testdata",
	}
	fileName = GetNewRecipeFilePath(recipeInfo, config)
	assert.Equal(t, "/tmp/out/recipe.md", fileName)

	// Test with OutputDir and nested subdirectory
	recipeInfo2 := types.Info{
		RecipeFilePath: "testdata/desserts/cake.cook",
		RecipeName:     "Cake",
	}
	fileName2 := GetNewRecipeFilePath(recipeInfo2, config)
	assert.Equal(t, "/tmp/out/desserts/cake.md", fileName2)
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

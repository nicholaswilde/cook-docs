package document

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/nicholaswilde/cook-docs/pkg/types"
	"github.com/aquilax/cooklang-go"
	"github.com/stretchr/testify/assert"
)

func TestPrintDocumentationCreatesDirectories(t *testing.T) {
	tempDir, err := os.MkdirTemp("", "cook-docs-test-*")
	assert.NoError(t, err)
	defer os.RemoveAll(tempDir)

	targetDir := filepath.Join(tempDir, "nested", "deeply")
	targetFilePath := filepath.Join(targetDir, "test-recipe.md")

	recipe := &types.Recipe{
		Steps:    []cooklang.Step{},
		Metadata: cooklang.Metadata{},
		Config: types.Config{
			DryRun:        false,
			TemplateFiles: []string{}, // No template files, will fall back to default
		},
		Info: types.Info{
			RecipeFilePath:    "testdata/Recipe.cook",
			RecipeName:        "Test Recipe",
			NewRecipeFilePath: targetFilePath,
		},
	}

	PrintDocumentation(recipe)

	// Verify that the file and directory are created
	_, err = os.Stat(targetFilePath)
	assert.NoError(t, err, "File should have been created successfully")

	_, err = os.Stat(targetDir)
	assert.NoError(t, err, "Target directory should have been created successfully")
}

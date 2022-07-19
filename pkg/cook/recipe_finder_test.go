package cook

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestFindRecipeFilePaths(t *testing.T) {
	recipeDirs, _ := FindRecipeFilePaths(".")
	assert.Equal(t, "testdata/Recipe.cook", recipeDirs[0])
}

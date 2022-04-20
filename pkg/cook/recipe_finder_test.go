package cook

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestFindRecipePaths(t *testing.T) {
	recipeDirs, _ := FindRecipePaths(".")
	assert.Equal(t, "testdata/Recipe.cook", recipeDirs[0])
}

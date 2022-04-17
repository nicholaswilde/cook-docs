package cook

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestFindRecipeDirectories(t *testing.T) {
	recipeDirs, err := FindRecipeDirectories(".")
	require.NoError(t, err)
	assert.Equal(t, "testdata/Recipe.cook", recipeDirs[0])
}

package util

import (
  "testing"

  "github.com/stretchr/testify/require"
)

func TestIsRelativePath(t *testing.T){
  require.False(t, IsRelativePath("Recipe.cook"))
  require.False(t, IsRelativePath("testdata/Recipe.cook"))
  require.True(t, IsRelativePath("./testdata/Recipe.cook"))
}

func TestIsBaseFilename(t *testing.T){
  require.False(t, IsBaseFilename("testdata/Recipe.cook"))
  require.False(t, IsBaseFilename("./testdata/Recipe.cook"))
  require.True(t, IsBaseFilename("Recipe.cook"))
}

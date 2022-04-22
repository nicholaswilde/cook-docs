package util

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestFindGitRepositoryRoot(t *testing.T) {
	root, err := FindGitRepositoryRoot()
	require.NoError(t, err)
	cwd, _ := os.Getwd()
	dir, _ := os.Open(filepath.Join(cwd, "../../"))
	assert.Equal(t, dir.Name(), root)
}

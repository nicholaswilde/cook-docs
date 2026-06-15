package main

import (
	"testing"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"github.com/stretchr/testify/assert"
	"github.com/nicholaswilde/cook-docs/pkg/types"
)

func TestCommandLineOutputDirFlag(t *testing.T) {
	viper.Reset()
	cmd, err := newCookDocsCommand(func(cmd *cobra.Command, args []string) {})
	assert.NoError(t, err)

	// Set arguments specifying the output directory
	cmd.SetArgs([]string{"--output-dir", "/tmp/recipes-out"})
	err = cmd.Execute()
	assert.NoError(t, err)

	var config types.Config
	err = viper.Unmarshal(&config)
	assert.NoError(t, err)

	assert.Equal(t, "/tmp/recipes-out", config.OutputDir)
}

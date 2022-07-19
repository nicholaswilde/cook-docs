package main

import (
	"os"
	"path"
	"strings"
	"sync"

	"github.com/nicholaswilde/cook-docs/pkg/cook"
	"github.com/nicholaswilde/cook-docs/pkg/document"
	"github.com/nicholaswilde/cook-docs/pkg/types"
	log "github.com/sirupsen/logrus"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

func retrieveInfoAndPrintDocumentation(recipePath string, waitGroup *sync.WaitGroup, config *types.Config) {
	defer waitGroup.Done()

	recipe, err := cook.ParseFile(recipePath, config)
	if err != nil {
		log.Warnf("Error parsing file for recipe %s, skipping: %s", recipePath, err)
		return
	}

	document.PrintDocumentation(recipe)
}

func GetFullSearchRoot(searchRoot string) (string, error) {
	var fullSearchRoot string
	if path.IsAbs(searchRoot) {
		fullSearchRoot = searchRoot
	} else {
		cwd, err := os.Getwd()
		if err != nil {
			return "", err
		}
		fullSearchRoot = path.Join(cwd, searchRoot)
	}
	return fullSearchRoot, nil
}

func cookDocs(_ *cobra.Command, _ []string) {
	var config types.Config
	viper.Unmarshal(&config)

	initializeCli(&config)

	fullSearchRoot, err := GetFullSearchRoot(config.RecipeSearchRoot)
	if err != nil {
		log.Warnf("Error getting working directory: %s", err)
		return
	}

	recipePaths, err := cook.FindRecipeFilePaths(fullSearchRoot)
	if err != nil {
		log.Errorf("Error finding recipe paths: %s", err)
		os.Exit(1)
	}
	log.Infof("Found recipes [%s]", strings.Join(recipePaths, ", "))

	log.Debugf("Rendering from optional template files [%s]", strings.Join(config.TemplateFiles, ", "))

	waitGroup := sync.WaitGroup{}

	for _, r := range recipePaths {
		waitGroup.Add(1)

		// On dry runs all output goes to stdout, and so as to not jumble things, generate serially
		if config.DryRun {
			retrieveInfoAndPrintDocumentation(r, &waitGroup, &config)
		} else {
			go retrieveInfoAndPrintDocumentation(r, &waitGroup, &config)
		}
	}
	waitGroup.Wait()
}

func main() {
	command, err := newCookDocsCommand(cookDocs)
	if err != nil {
		log.Errorf("Failed to create the CLI commander: %s", err)
		os.Exit(1)
	}
	if err := command.Execute(); err != nil {
		log.Errorf("Failed to start the CLI: %s", err)
		os.Exit(1)
	}
}

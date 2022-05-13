package main

import (
	"os"
	"path"
	"strings"
	"sync"

	"github.com/aquilax/cooklang-go"
	"github.com/nicholaswilde/cook-docs/pkg/cook"
	"github.com/nicholaswilde/cook-docs/pkg/document"
	"github.com/nicholaswilde/cook-docs/pkg/types"
	log "github.com/sirupsen/logrus"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

func retrieveInfoAndPrintDocumentation(recipeSearchRoot string, recipePath string, templateFiles []string, waitGroup *sync.WaitGroup, config *types.Config) {
	defer waitGroup.Done()

	recipeInfo := cook.ParseRecipeInformation(recipePath)

	recipeData, err := cooklang.ParseFile(recipeInfo.RecipePath)

	if err != nil {
		log.Warnf("Error parsing file for recipe %s, skipping: %s", recipeInfo.RecipePath, err)
		return
	}

	recipeData = cook.MergeRecipeData(recipeInfo, recipeData)

	document.PrintDocumentation(recipeSearchRoot, recipeData, recipeInfo, templateFiles, config)
}

func cookDocs(_ *cobra.Command, _ []string) {
	var config types.Config
	viper.Unmarshal(&config)
	log.Println(config)
	
	initializeCli(&config)

	recipeSearchRoot := config.RecipeSearchRoot
	
	var fullRecipeSearchRoot string
	if path.IsAbs(recipeSearchRoot) {
		fullRecipeSearchRoot = recipeSearchRoot
	} else {
		cwd, err := os.Getwd()
		if err != nil {
			log.Warnf("Error getting working directory: %s", err)
			return
		}
		fullRecipeSearchRoot = path.Join(cwd, recipeSearchRoot)
	}

	recipePaths, err := cook.FindRecipePaths(fullRecipeSearchRoot)
	if err != nil {
		log.Errorf("Error finding recipe paths: %s", err)
		os.Exit(1)
	}
	log.Infof("Found recipes [%s]", strings.Join(recipePaths, ", "))

	templateFiles := config.TemplateFiles
	log.Debugf("Rendering from optional template files [%s]", strings.Join(templateFiles, ", "))

	waitGroup := sync.WaitGroup{}

	for _, r := range recipePaths {
		waitGroup.Add(1)

		// On dry runs all output goes to stdout, and so as to not jumble things, generate serially
		if config.DryRun {
			retrieveInfoAndPrintDocumentation(fullRecipeSearchRoot, r, templateFiles, &waitGroup, &config)
		} else {
			go retrieveInfoAndPrintDocumentation(fullRecipeSearchRoot, r, templateFiles, &waitGroup, &config)
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

package main

import (
  "os"
  "path"
  "sync"
  "strings"

  "github.com/aquilax/cooklang-go"
  log "github.com/sirupsen/logrus"
  "github.com/spf13/cobra"
  "github.com/spf13/viper"
  "github.com/nicholaswilde/cook-docs/pkg/cook"
  "github.com/nicholaswilde/cook-docs/pkg/document"
)

func retrieveInfoAndPrintDocumentation(recipeSearchRoot string, recipePath string, templateFiles []string, waitGroup *sync.WaitGroup) {
  defer waitGroup.Done()

  recipeInfo := cook.ParseRecipeInformation(recipePath)

	recipeData, err := cooklang.ParseFile(recipeInfo.RecipePath)

	if err != nil {
    log.Warnf("Error parsing file for recipe %s, skipping: %s", recipeInfo.RecipePath, err)
		return
	}

  recipeData = cook.MergeRecipeData(recipeInfo, recipeData)

  document.PrintDocumentation(recipeSearchRoot, recipeData, recipeInfo, templateFiles)
}

func cookDocs(_ *cobra.Command, _ []string) {
	initializeCli()
	recipeSearchRoot := viper.GetString("recipe-search-root")
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
  recipeDirs, err := cook.FindRecipeDirectories(fullRecipeSearchRoot)
  if err != nil {
		log.Errorf("Error finding recipe directories: %s", err)
		os.Exit(1)
	}
  log.Infof("Found recipes [%s]", strings.Join(recipeDirs, ", "))

  templateFiles := viper.GetStringSlice("template-files")
	log.Debugf("Rendering from optional template files [%s]", strings.Join(templateFiles, ", "))

  waitGroup := sync.WaitGroup{}

  for _, r := range recipeDirs {
    waitGroup.Add(1)
    retrieveInfoAndPrintDocumentation(fullRecipeSearchRoot, r, templateFiles, &waitGroup)
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

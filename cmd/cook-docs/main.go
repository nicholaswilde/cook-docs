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

func retrieveInfoAndPrintDocumentation(recipePath string, waitGroup *sync.WaitGroup) {
  defer waitGroup.Done()
  _, _ = cook.ParseRecipeInformation(recipePath)
	r, err := cooklang.ParseFile(recipePath)
	if err != nil {
    log.Warnf("Error parsing information for recipe %s, skipping: %s", recipePath, err)
		return
	}
  document.PrintDocumentation(recipePath, r)
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
  waitGroup := sync.WaitGroup{}

  for _, r := range recipeDirs {
    waitGroup.Add(1)
    retrieveInfoAndPrintDocumentation(r, &waitGroup)
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

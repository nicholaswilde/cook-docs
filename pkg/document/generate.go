package document

import (
	"bytes"
	"encoding/json"
	"os"
	"regexp"

	"github.com/nicholaswilde/cook-docs/pkg/types"
	log "github.com/sirupsen/logrus"
)

func getOutputFile(newFileName string, dryRun bool) (*os.File, error) {
	if dryRun {
		return os.Stdout, nil
	}
	return os.Create(newFileName)
}

func applyMarkDownFormat(output bytes.Buffer) bytes.Buffer {
	outputString := output.String()
	re := regexp.MustCompile(` \n`)
	outputString = re.ReplaceAllString(outputString, "\n")

	re = regexp.MustCompile(`\n{3,}`)
	outputString = re.ReplaceAllString(outputString, "\n\n")

	output.Reset()
	output.WriteString(outputString)
	return output
}

func PrintDocumentation(recipe *types.Recipe) {
	
	if recipe.Config.Jsonify {
		log.Infof("Printing json output for recipe %s", recipe.Info.NewRecipeFilePath)
		j, err := json.MarshalIndent(recipe, "", "  ")
		if err != nil {
			log.Fatal(err)
		}
		log.Info(string(j))
		return
	}

	log.Infof("Generating markdown file for recipe %s", recipe.Info.NewRecipeFilePath)

	t, err := newRecipeDocumentationTemplate(recipe)
	if err != nil {
		log.Warnf("Error getting template data %s: %s", recipe.Info.RecipeFilePath, err)
		return
	}

	outputFile, err := getOutputFile(recipe.Info.NewRecipeFilePath, recipe.Config.DryRun)
	if err != nil {
		log.Warnf("Could not open recipe markdown file %s, skipping recipe: %s", recipe.Info.NewRecipeFilePath, err)
		return
	}

	if !recipe.Config.DryRun {
		defer outputFile.Close()
	}

	var output bytes.Buffer
	err = t.Execute(&output, recipe)
	if err != nil {
		log.Warnf("Error executing template %s: %s", recipe.Info.RecipeFilePath, err)
	}

	output = applyMarkDownFormat(output)
	_, err = output.WriteTo(outputFile)
	if err != nil {
		log.Warnf("Error generating documentation file for recipe %s: %s", recipe.Info.NewRecipeFilePath, err)
	}
}

package document

import (
	"bytes"
	"encoding/json"
	"os"
	"regexp"

	"github.com/aquilax/cooklang-go"
	"github.com/nicholaswilde/cook-docs/pkg/cook"
	"github.com/nicholaswilde/cook-docs/pkg/types"
	log "github.com/sirupsen/logrus"
)

func getOutputFile(recipeInfo cook.RecipeDocumentationInfo, dryRun bool) (*os.File, error) {
	if dryRun {
		return os.Stdout, nil
	}
	log.Debug(recipeInfo.NewFileName)
	return os.Create(recipeInfo.NewFileName)
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

func PrintDocumentation(recipeSearchRoot string, recipeData *cooklang.Recipe, recipeInfo cook.RecipeDocumentationInfo, templateFiles []string, config *types.Config) {
	
	if config.Jsonify {
		log.Infof("Printing json output for recipe %s", recipeInfo.NewFileName)
		j, err := json.MarshalIndent(recipeData, "", "  ")
		if err != nil {
			log.Fatal(err)
		}
		log.Info(string(j))
		return
	}

	log.Infof("Generating markdown file for recipe %s", recipeInfo.NewFileName)

	t, err := newRecipeDocumentationTemplate(recipeSearchRoot, recipeInfo, templateFiles, config)
	if err != nil {
		log.Warnf("Error getting template data %s: %s", recipeInfo.RecipePath, err)
		return
	}

	outputFile, err := getOutputFile(recipeInfo, config.DryRun)
	if err != nil {
		log.Warnf("Could not open recipe markdown file %s, skipping recipe: %s", recipeInfo.NewFileName, err)
		return
	}

	if !config.DryRun {
		defer outputFile.Close()
	}

	var output bytes.Buffer
	err = t.Execute(&output, recipeData)
	if err != nil {
		log.Warnf("Error executing template %s: %s", recipeInfo.RecipePath, err)
	}

	output = applyMarkDownFormat(output)
	_, err = output.WriteTo(outputFile)
	if err != nil {
		log.Warnf("Error generating documentation file for recipe %s: %s", recipeInfo.NewFileName, err)
	}
}

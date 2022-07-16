package document

import (
	"errors"
	"fmt"
	"io/ioutil"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"text/template"

	"github.com/Masterminds/sprig/v3"
	"github.com/aquilax/cooklang-go"
	"github.com/nicholaswilde/cook-docs/pkg/cook"
	"github.com/nicholaswilde/cook-docs/pkg/util"
	log "github.com/sirupsen/logrus"
)

const defaultDocumentationTemplate = `{{ template "cook.headerSection" . }}

{{ template "cook.imageSection" . }}

{{ template "cook.tableSection" . }}

{{ template "cook.ingredientsSection" . }}

{{ template "cook.cookwareSection" . }}

{{ template "cook.stepsSection" . -}}

{{ template "cook.sourceSection" . }}
`

func getHeaderTemplate() string {
	templateBuilder := strings.Builder{}

	templateBuilder.WriteString(`{{ define "cook.headerSection" }}`)
	templateBuilder.WriteString("# {{ .Metadata.title }}")
	templateBuilder.WriteString("{{ end }}")

	return templateBuilder.String()
}

func getImageTemplate() string {
	templateBuilder := strings.Builder{}

	templateBuilder.WriteString(`{{ define "cook.imageSection" }}`)
	templateBuilder.WriteString("{{ if .Metadata.ImageName }}")
	templateBuilder.WriteString(`![{{ .Metadata.title }}](../assets/images/{{ lower .Metadata.ImageName | replace " " "-" }})`)
	templateBuilder.WriteString("{{ end }}")
	templateBuilder.WriteString("{{ end }}")

	return templateBuilder.String()
}

func getTableTemplate() string {
	templateBuilder := strings.Builder{}

	templateBuilder.WriteString(`{{ define "cook.tableSection" }}`)
	templateBuilder.WriteString("{{ if or .Metadata.servings .Metadata.serves }}")
	templateBuilder.WriteString("| :fork_and_knife_with_plate: Serves | :timer_clock: Total Time |\n")
	templateBuilder.WriteString("|:----------------------------------:|:-----------------------: |\n")
	templateBuilder.WriteString("| {{ if .Metadata.servings }}{{ .Metadata.servings }}{{ else if .Metadata.serves }}{{ .Metadata.serves }}{{ end }} | {{ sumTimers .Steps }} |")
	templateBuilder.WriteString("{{ else }}")
	templateBuilder.WriteString("| :timer_clock: Total Time |\n")
	templateBuilder.WriteString("|:-----------------------: |\n")
	templateBuilder.WriteString("| {{ sumTimers .Steps }} |")
	templateBuilder.WriteString("{{ end }}")
	templateBuilder.WriteString("{{ end }}")

	return templateBuilder.String()
}

func getIngredientsTemplate() string {
	templateBuilder := strings.Builder{}

	templateBuilder.WriteString(`{{ define "cook.ingredientsHeader" }}`)
	templateBuilder.WriteString("## :salt: Ingredients")
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{ define "cook.ingredients" }}`)
	templateBuilder.WriteString("{{ range .Steps }}")
	templateBuilder.WriteString("{{- range .Ingredients }}")
	templateBuilder.WriteString("\n")
	templateBuilder.WriteString("- {{ if .Amount.Quantity }}{{ round .Amount.Quantity 2 }}{{ if .Amount.Unit }} {{ .Amount.Unit }}{{ end }}{{ else }}some{{ end }}")
	templateBuilder.WriteString(" {{ .Name }}")
	templateBuilder.WriteString("{{- end }}")
	templateBuilder.WriteString("{{- end }}")
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{ define "cook.ingredientsSection" }}`)
	templateBuilder.WriteString(`{{ template "cook.ingredientsHeader" . }}`)
	templateBuilder.WriteString("\n")
	templateBuilder.WriteString(`{{ template "cook.ingredients" . }}`)
	templateBuilder.WriteString("{{ end }}")

	return templateBuilder.String()
}

func getCookwareTemplate() string {
	templateBuilder := strings.Builder{}

	templateBuilder.WriteString(`{{ define "cook.cookwareHeader" }}`)
	templateBuilder.WriteString("## :cooking: Cookware")
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{ define "cook.cookware" }}`)
	templateBuilder.WriteString("{{ range .Steps }}{{- range .Cookware }}")
	templateBuilder.WriteString("\n")
	templateBuilder.WriteString("- {{.Quantity}} {{.Name}}{{- end }}{{- end }}")
	templateBuilder.WriteString("{{ end }}")
	templateBuilder.WriteString(`{{ define "cook.cookwareSection" }}`)
	templateBuilder.WriteString(`{{ template "cook.cookwareHeader" . }}`)
	templateBuilder.WriteString("\n")
	templateBuilder.WriteString(`{{ template "cook.cookware" . }}`)
	templateBuilder.WriteString("{{ end }}")

	return templateBuilder.String()
}

func getStepsTemplate() string {
	templateBuilder := strings.Builder{}

	templateBuilder.WriteString(`{{ define "cook.stepsHeader" }}`)
	templateBuilder.WriteString("## :pencil: Instructions")
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{ define "cook.steps" }}`)
	templateBuilder.WriteString("{{ range $i, $a := .Steps }}\n\n### Step {{add1 $i}}\n\n{{ wrap 120 .Directions }}{{- end }}")
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{ define "cook.stepsSection" }}`)
	templateBuilder.WriteString(`{{ template "cook.stepsHeader" . }}`)
	templateBuilder.WriteString(`{{ template "cook.steps" . }}`)
	templateBuilder.WriteString("{{ end }}")

	return templateBuilder.String()
}

func getStepsWithQuotedCommentsTemplate() string {
	templateBuilder := strings.Builder{}

	templateBuilder.WriteString(`{{ define "cook.stepsWithQuotedCommentsHeader" }}`)
	templateBuilder.WriteString("## :pencil: Instructions")
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{ define "cook.stepsWithQuotedComments" }}`)
	templateBuilder.WriteString("{{ range $i, $a := .Steps }}")
	templateBuilder.WriteString("\n\n### Step {{add1 $i}}")
	templateBuilder.WriteString("\n\n{{ wrap 120 .Directions }}")
	templateBuilder.WriteString("\n\n{{ range .Comments }}\n> {{.}}{{- end }}")
	templateBuilder.WriteString("{{- end }}")
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{ define "cook.stepsWithQuotedCommentsSection" }}`)
	templateBuilder.WriteString(`{{ template "cook.stepsWithQuotedCommentsHeader" . }}`)
	templateBuilder.WriteString(`{{ template "cook.stepsWithQuotedComments" . }}`)
	templateBuilder.WriteString("{{ end }}")

	return templateBuilder.String()
}

func getStepsWithAdmonishedCommentsTemplate() string {
	templateBuilder := strings.Builder{}

	templateBuilder.WriteString(`{{ define "cook.stepsWithAdmonishedCommentsHeader" }}`)
	templateBuilder.WriteString("## :pencil: Instructions")
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{ define "cook.stepsWithAdmonishedComments" }}`)
	templateBuilder.WriteString("{{ range $i, $a := .Steps }}")
	templateBuilder.WriteString("\n\n### Step {{add1 $i}}")
	templateBuilder.WriteString("\n\n{{ wrap 120 .Directions }}")
	templateBuilder.WriteString("\n\n{{ range .Comments }}")
	templateBuilder.WriteString("\n!!! note")
	templateBuilder.WriteString("\n{{ indent 6 . }}")
	templateBuilder.WriteString("{{- end }}")
	templateBuilder.WriteString("{{- end }}")
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{ define "cook.stepsWithAdmonishedCommentsSection" }}`)
	templateBuilder.WriteString(`{{ template "cook.stepsWithAdmonishedCommentsHeader" . }}`)
	templateBuilder.WriteString(`{{ template "cook.stepsWithAdmonishedComments" . }}`)
	templateBuilder.WriteString("{{ end }}")

	return templateBuilder.String()
}

func getSourceTemplate() string {
	templateBuilder := strings.Builder{}

	templateBuilder.WriteString(`{{ define "cook.sourceHeader" }}`)
	templateBuilder.WriteString("## :link: Source")
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{- define "cook.source" }}`)
	templateBuilder.WriteString("- {{ getSource .Metadata.source }}")
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{ define "cook.sourceSection" }}`)
	templateBuilder.WriteString("{{ if .Metadata.source }}")
	templateBuilder.WriteString("\n\n")
	templateBuilder.WriteString(`{{ template "cook.sourceHeader" . }}`)
	templateBuilder.WriteString("\n\n")
	templateBuilder.WriteString(`{{ template "cook.source" . }}`)
	templateBuilder.WriteString("{{ end }}")
	templateBuilder.WriteString("{{ end }}")

	return templateBuilder.String()
}

func getMetadataTemplate() string {
	templateBuilder := strings.Builder{}

	templateBuilder.WriteString(`{{ define "cook.metadataHeader" }}`)
	templateBuilder.WriteString("## Metadata")
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{ define "cook.metadata" }}`)
	templateBuilder.WriteString(`{{ range $key, $value := .Metadata }}\n- {{ $key }}: {{ $value }}{{ end }}`)
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{ define "cook.metadataSection" }}`)
	templateBuilder.WriteString(`{{ template "cook.metadataHeader" . }}`)
	templateBuilder.WriteString("\n")
	templateBuilder.WriteString(`{{ template "cook.metadata" . }}`)
	templateBuilder.WriteString("{{ end }}")

	return templateBuilder.String()
}

func getCommentsTemplate() string {
	templateBuilder := strings.Builder{}

	templateBuilder.WriteString(`{{ define "cook.commentsHeader" }}`)
	templateBuilder.WriteString("## Comments")
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{ define "cook.comments" }}`)
	templateBuilder.WriteString("{{ range .Steps }}{{ range .Comments }}\n- {{.}}{{- end }}{{- end }}")
	templateBuilder.WriteString("{{ end }}")

	templateBuilder.WriteString(`{{ define "cook.commentsSection" }}`)
	templateBuilder.WriteString(`{{ template "cook.commentsHeader" . }}`)
	templateBuilder.WriteString("\n")
	templateBuilder.WriteString(`{{ template "cook.comments" . }}`)
	templateBuilder.WriteString("{{ end }}")

	return templateBuilder.String()
}

func getDocumentationTemplate(recipeSearchRoot string, recipePath string, templateFiles []string) (string, error) {
	templateFilesForRecipe := make([]string, 0)

	var templateNotFound bool

	for _, templateFile := range templateFiles {
		var fullTemplatePath string

		if util.IsRelativePath(templateFile) {
			fullTemplatePath = filepath.Join(recipeSearchRoot, templateFile)
		} else if util.IsBaseFilename(templateFile) {
			fullTemplatePath = filepath.Join(filepath.Dir(recipePath), templateFile)
		} else {
			fullTemplatePath = templateFile
		}

		_, err := os.Stat(fullTemplatePath)
		if errors.Is(err, os.ErrNotExist) {
			log.Debugf("Did not find template file %s for recipe %s, using default template", fullTemplatePath, recipePath)

			templateNotFound = true
			continue
		}

		templateFilesForRecipe = append(templateFilesForRecipe, fullTemplatePath)
	}

	log.Debugf("Using template files %s for chart %s", templateFiles, recipePath)
	allTemplateContents := make([]byte, 0)

	for _, templateFileForRecipe := range templateFilesForRecipe {
		templateContents, err := ioutil.ReadFile(templateFileForRecipe)
		if err != nil {
			return "", err
		}
		allTemplateContents = append(allTemplateContents, templateContents...)
	}

	if templateNotFound {
		allTemplateContents = append(allTemplateContents, []byte(defaultDocumentationTemplate)...)
	}

	return string(allTemplateContents), nil
}

func getDocumentationTemplates(recipeSearchRoot string, recipePath string, templateFiles []string) ([]string, error) {
	documentationTemplate, err := getDocumentationTemplate(recipeSearchRoot, recipePath, templateFiles)

	if err != nil {
		log.Errorf("Failed to read documentation template for recipe %s: %s", recipePath, err)
		return nil, err
	}

	return []string{
		getHeaderTemplate(),
		getImageTemplate(),
		getTableTemplate(),
		getIngredientsTemplate(),
		getCookwareTemplate(),
		getStepsTemplate(),
		getStepsWithQuotedCommentsTemplate(),
		getStepsWithAdmonishedCommentsTemplate(),
		getSourceTemplate(),
		getMetadataTemplate(),
		getCommentsTemplate(),
		documentationTemplate,
	}, nil
}

func newRecipeDocumentationTemplate(recipeSearchRoot string, recipeInfo cook.RecipeDocumentationInfo, templateFiles []string) (*template.Template, error) {
	documentationTemplate := template.New(recipeInfo.RecipePath)
	documentationTemplate.Funcs(sprig.TxtFuncMap())
	documentationTemplate.Funcs(template.FuncMap{"getSource": func(source string) string {
		_, err := url.ParseRequestURI(source)
		if err != nil {
			return source
		}
		u, err := url.Parse(source)
		if err != nil || u.Scheme == "" || u.Host == "" {
			return source
		}
		return "<" + source + ">"
	}})
	documentationTemplate.Funcs(template.FuncMap{"sumTimers": func(steps []cooklang.Step) string {
		var sum float64
		for _, s := range steps {
			for _, t := range s.Timers {
				switch t.Unit {
				case "day", "days":
					sum = sum + t.Duration*60*24
				case "hour", "hours":
					sum = sum + t.Duration*60
				case "minute", "minutes":
					sum += t.Duration
				}
			}
		}
		if sum > 1440 {
			sum = sum / 1440
			return fmt.Sprintf("%.2f days", sum)
		} else if sum > 60 {
			sum = sum / 60
			return fmt.Sprintf("%.2f hours", sum)
		} else {
			return fmt.Sprintf("%.0f minutes", sum)
		}
	}})

	goTemplateList, err := getDocumentationTemplates(recipeSearchRoot, recipeInfo.RecipePath, templateFiles)
	if err != nil {
		return nil, err
	}
	for _, t := range goTemplateList {
		_, err := documentationTemplate.Parse(t)

		if err != nil {
			return nil, err
		}
	}

	return documentationTemplate, nil
}

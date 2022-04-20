package main

import (
	"fmt"
	"os"
	"strings"

	log "github.com/sirupsen/logrus"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var version string

func possibleLogLevels() []string {
	levels := make([]string, 0)

	for _, l := range log.AllLevels {
		levels = append(levels, l.String())
	}

	return levels
}

func initializeCli() {
	logLevelName := viper.GetString("log-level")
	logLevel, err := log.ParseLevel(logLevelName)
	if err != nil {
		log.Errorf("Failed to parse provided log level %s: %s", logLevelName, err)
		os.Exit(1)
	}

	log.SetFormatter(&log.TextFormatter{FullTimestamp: true})
	log.SetLevel(logLevel)
}

func newCookDocsCommand(run func(cmd *cobra.Command, args []string)) (*cobra.Command, error) {
	command := &cobra.Command{
		Use:     "cook-docs",
		Short:   "cook-docs automatically generates markdown documentation for cook recipes from template files",
		Version: version,
		Run:     run,
	}
	logLevelUsage := fmt.Sprintf("level of logs that should printed, one of (%s)", strings.Join(possibleLogLevels(), ", "))
	command.PersistentFlags().BoolP("dry-run", "d", false, "don't actually render any markdown files just print to stdout passed")
	command.PersistentFlags().BoolP("jsonify", "j", false, "parse the recipe and display it in json format")
	command.PersistentFlags().StringP("ignore-file", "i", ".cookdocsignore", "filename to use as an ignore file to exclude recipe directories")
	command.PersistentFlags().StringP("recipe-search-root", "c", ".", "directory to search recursively within for recipes")
	command.PersistentFlags().StringP("log-level", "l", "info", logLevelUsage)
	command.PersistentFlags().StringSliceP("template-files", "t", []string{"recipe.md.gotmpl"}, "gotemplate file paths relative to each recipe directory from which documentation will be generated")

	viper.SetConfigName(".cookdocs")
	viper.SetConfigType("yaml")
	viper.AddConfigPath("/etc/cook-docs/")
	viper.AddConfigPath("$HOME/.config/")
	viper.AddConfigPath(".")
	err := viper.ReadInConfig()
	if err != nil {
		if _, ok := err.(viper.ConfigFileNotFoundError); ok {
			// Config file not found; ignore error if desired
			log.Debug(err)
		} else {
			log.Debugf("Error occured while reading config file: %s \n", err)
		}
	} else {
		log.Debugf("Using config file %s", viper.ConfigFileUsed())
	}

	viper.AutomaticEnv()
	viper.SetEnvPrefix("COOK_DOCS")
	viper.SetEnvKeyReplacer(strings.NewReplacer("-", "_"))
	err = viper.BindPFlags(command.PersistentFlags())
	return command, err
}

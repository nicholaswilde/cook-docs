package types

import "github.com/aquilax/cooklang-go"

type Config struct {
	DryRun				bool		`mapstructure:"dry-run"`
	Jsonify				bool		`mapstructure:"jsonify"`
	IgnoreFile			string		`mapstructure:"ignore-file"`
	RecipeSearchRoot	string		`mapstructure:"recipe-search-root"`
	LogLevel			string		`mapstructure:"log-level"`
	TemplateFiles		[]string	`mapstructure:"template-files"`
	WordWrap			int			`mapstructure:"word-wrap"`
}

// Recipe contains a cooklang defined recipe
type Recipe struct {
	Steps    	[]cooklang.Step   // list of steps for the recipe
	Metadata 	cooklang.Metadata // metadata of the recipe
	Config		Config
	Info		Info
}

type Info struct {
	ImagePath	string
	ImageName	string
	RecipeName	string
	RecipePath	string
	NewFileName	string
}

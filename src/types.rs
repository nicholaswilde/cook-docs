use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "DryRun", alias = "dry-run", alias = "dry_run", alias = "dryRun")]
    pub dry_run: bool,
    #[serde(rename = "Jsonify", alias = "jsonify")]
    pub jsonify: bool,
    #[serde(rename = "IgnoreFile", alias = "ignore-file", alias = "ignore_file", alias = "ignoreFile")]
    pub ignore_file: String,
    #[serde(rename = "RecipeSearchRoot", alias = "recipe-search-root", alias = "recipe_search_root", alias = "recipeSearchRoot")]
    pub recipe_search_root: String,
    #[serde(rename = "LogLevel", alias = "log-level", alias = "log_level", alias = "logLevel")]
    pub log_level: String,
    #[serde(rename = "TemplateFiles", alias = "template-files", alias = "template_files", alias = "templateFiles")]
    pub template_files: Vec<String>,
    #[serde(rename = "WordWrap", alias = "word-wrap", alias = "word_wrap", alias = "wordWrap")]
    pub word_wrap: usize,
    #[serde(rename = "OutputDir", alias = "output-dir", alias = "output_dir", alias = "outputDir")]
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientAmount {
    #[serde(rename = "IsNumeric")]
    pub is_numeric: bool,
    #[serde(rename = "Quantity")]
    pub quantity: f64,
    #[serde(rename = "QuantityRaw")]
    pub quantity_raw: String,
    #[serde(rename = "Unit")]
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingredient {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Amount")]
    pub amount: IngredientAmount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookware {
    #[serde(rename = "IsNumeric")]
    pub is_numeric: bool,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Quantity")]
    pub quantity: f64,
    #[serde(rename = "QuantityRaw")]
    pub quantity_raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Duration")]
    pub duration: f64,
    #[serde(rename = "Unit")]
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    #[serde(rename = "Directions")]
    pub directions: String,
    #[serde(rename = "Timers")]
    pub timers: Vec<Timer>,
    #[serde(rename = "Ingredients")]
    pub ingredients: Vec<Ingredient>,
    #[serde(rename = "Cookware")]
    pub cookware: Vec<Cookware>,
    #[serde(rename = "Comments")]
    pub comments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    #[serde(rename = "ImageFileName")]
    pub image_file_name: String,
    #[serde(rename = "ImageFilePath")]
    pub image_file_path: String,
    #[serde(rename = "NewRecipeFilePath")]
    pub new_recipe_file_path: String,
    #[serde(rename = "RecipeName")]
    pub recipe_name: String,
    #[serde(rename = "RecipeFilePath")]
    pub recipe_file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    #[serde(rename = "Steps")]
    pub steps: Vec<Step>,
    #[serde(rename = "Metadata")]
    pub metadata: HashMap<String, String>,
    #[serde(rename = "Config")]
    pub config: Config,
    #[serde(rename = "Info")]
    pub info: Info,
}

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrVec {
    One(String),
    Many(Vec<String>),
}

pub(crate) fn deserialize_category_tags<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = BTreeMap::<String, StringOrVec>::deserialize(deserializer)?;
    Ok(raw
        .into_iter()
        .map(|(k, v)| {
            let tags = match v {
                StringOrVec::One(s) => vec![s],
                StringOrVec::Many(list) => list,
            };
            (k, tags)
        })
        .collect())
}

pub(crate) fn deserialize_opt_category_tags<'de, D>(
    deserializer: D,
) -> Result<Option<BTreeMap<String, Vec<String>>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = Option::<BTreeMap<String, StringOrVec>>::deserialize(deserializer)?;
    Ok(raw.map(|map| {
        map.into_iter()
            .map(|(k, v)| {
                let tags = match v {
                    StringOrVec::One(s) => vec![s],
                    StringOrVec::Many(list) => list,
                };
                (k, tags)
            })
            .collect()
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(
        rename = "DryRun",
        alias = "dry-run",
        alias = "dry_run",
        alias = "dryRun"
    )]
    pub dry_run: bool,
    #[serde(rename = "Jsonify", alias = "jsonify")]
    pub jsonify: bool,
    #[serde(
        rename = "IgnoreFile",
        alias = "ignore-file",
        alias = "ignore_file",
        alias = "ignoreFile"
    )]
    pub ignore_file: String,
    #[serde(
        rename = "RecipeSearchRoot",
        alias = "recipe-search-root",
        alias = "recipe_search_root",
        alias = "recipeSearchRoot"
    )]
    pub recipe_search_root: String,
    #[serde(
        rename = "LogLevel",
        alias = "log-level",
        alias = "log_level",
        alias = "logLevel"
    )]
    pub log_level: String,
    #[serde(
        rename = "TemplateFiles",
        alias = "template-files",
        alias = "template_files",
        alias = "templateFiles"
    )]
    pub template_files: Vec<String>,
    #[serde(
        rename = "WordWrap",
        alias = "word-wrap",
        alias = "word_wrap",
        alias = "wordWrap"
    )]
    pub word_wrap: usize,
    #[serde(
        rename = "OutputDir",
        alias = "output-dir",
        alias = "output_dir",
        alias = "outputDir"
    )]
    pub output_dir: String,
    #[serde(
        rename = "AssetsDir",
        alias = "assets-dir",
        alias = "assets_dir",
        alias = "assetsDir",
        default
    )]
    pub assets_dir: String,
    #[serde(
        rename = "ImageExt",
        alias = "image-ext",
        alias = "image_ext",
        alias = "imageExt",
        default
    )]
    pub image_ext: String,
    #[serde(
        rename = "EmojiFile",
        alias = "emoji-file",
        alias = "emoji_file",
        alias = "emojiFile",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub emoji_file: String,
    #[serde(
        rename = "Emoji",
        alias = "emoji",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub emoji: Option<serde_json::Value>,
    #[serde(
        rename = "WeightsFile",
        alias = "weights-file",
        alias = "weights_file",
        alias = "weightsFile",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub weights_file: String,
    #[serde(
        rename = "LinkDirs",
        alias = "link-dirs",
        alias = "link_dirs",
        alias = "linkDirs",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub link_dirs: Vec<String>,
    #[serde(
        rename = "CategoryTags",
        alias = "category-tags",
        alias = "category_tags",
        alias = "categoryTags",
        default,
        deserialize_with = "deserialize_category_tags",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub category_tags: BTreeMap<String, Vec<String>>,
    #[serde(
        rename = "RequiredMetadata",
        alias = "required-metadata",
        alias = "required_metadata",
        alias = "requiredMetadata",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub required_metadata: Vec<String>,
    #[serde(
        rename = "Merge",
        alias = "merge",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub merge: bool,
    #[serde(
        rename = "Force",
        alias = "force",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub force: bool,
    #[serde(
        rename = "RecipeFiles",
        alias = "recipe-files",
        alias = "recipe_files",
        alias = "recipeFiles",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub recipe_files: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dry_run: false,
            jsonify: false,
            ignore_file: ".cookdocsignore".to_string(),
            recipe_search_root: ".".to_string(),
            log_level: "info".to_string(),
            template_files: vec!["recipe.md.gotmpl".to_string()],
            word_wrap: 120,
            output_dir: String::new(),
            assets_dir: String::new(),
            image_ext: String::new(),
            emoji_file: String::new(),
            emoji: None,
            weights_file: String::new(),
            link_dirs: Vec::new(),
            category_tags: BTreeMap::new(),
            required_metadata: Vec::new(),
            merge: false,
            force: false,
            recipe_files: Vec::new(),
        }
    }
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
    #[serde(rename = "ImageSlug", default)]
    pub image_slug: String,
    #[serde(rename = "NewRecipeFilePath")]
    pub new_recipe_file_path: String,
    #[serde(rename = "RecipeName")]
    pub recipe_name: String,
    #[serde(rename = "RecipeSlug", default)]
    pub recipe_slug: String,
    #[serde(rename = "RecipeFilePath")]
    pub recipe_file_path: String,
    #[serde(rename = "RelRoot", default)]
    pub rel_root: String,
    #[serde(rename = "Categories", default)]
    pub categories: Vec<String>,
    #[serde(rename = "DefaultTags", default)]
    pub default_tags: Vec<String>,
    #[serde(rename = "Tags", default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    #[serde(rename = "Steps")]
    pub steps: Vec<Step>,
    #[serde(rename = "Ingredients", default)]
    pub ingredients: Vec<Ingredient>,
    #[serde(rename = "Cookware", default)]
    pub cookware: Vec<Cookware>,
    #[serde(rename = "Metadata")]
    pub metadata: HashMap<String, String>,
    #[serde(rename = "Frontmatter", default)]
    pub frontmatter: BTreeMap<String, serde_json::Value>,
    #[serde(rename = "Config")]
    pub config: Config,
    #[serde(rename = "Info")]
    pub info: Info,
}

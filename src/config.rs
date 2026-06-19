use std::path::PathBuf;
use figment::{Figment, providers::{Format, Yaml}};
use serde::Deserialize;
use clap::{Arg, Command, ArgAction};
use crate::types::Config;

#[derive(Deserialize, Debug)]
struct ConfigFile {
    #[serde(alias = "dry-run", alias = "dry_run", alias = "dryRun")]
    pub dry_run: Option<bool>,
    #[serde(alias = "jsonify")]
    pub jsonify: Option<bool>,
    #[serde(alias = "ignore-file", alias = "ignore_file", alias = "ignoreFile")]
    pub ignore_file: Option<String>,
    #[serde(alias = "recipe-search-root", alias = "recipe_search_root", alias = "recipeSearchRoot")]
    pub recipe_search_root: Option<String>,
    #[serde(alias = "log-level", alias = "log_level", alias = "logLevel")]
    pub log_level: Option<String>,
    #[serde(alias = "template-files", alias = "template_files", alias = "templateFiles")]
    pub template_files: Option<Vec<String>>,
    #[serde(alias = "word-wrap", alias = "word_wrap", alias = "wordWrap")]
    pub word_wrap: Option<usize>,
    #[serde(alias = "output-dir", alias = "output_dir", alias = "outputDir")]
    pub output_dir: Option<String>,
}

pub fn load_config(args: &[&str]) -> Result<Config, String> {
    // 1. Set default configuration values
    let mut config = Config {
        dry_run: false,
        jsonify: false,
        ignore_file: ".cookdocsignore".to_string(),
        recipe_search_root: ".".to_string(),
        log_level: "info".to_string(),
        template_files: vec!["recipe.md.gotmpl".to_string()],
        word_wrap: 120,
        output_dir: "".to_string(),
    };

    // 2. Resolve the config file path if it exists
    let mut config_paths = Vec::new();
    config_paths.push(PathBuf::from("/etc/cook-docs/.cookdocs.yaml"));
    config_paths.push(PathBuf::from("/etc/cook-docs/.cookdocs.yml"));

    if let Ok(home) = std::env::var("HOME") {
        let home_path = PathBuf::from(home);
        config_paths.push(home_path.join(".config/.cookdocs.yaml"));
        config_paths.push(home_path.join(".config/.cookdocs.yml"));
    }

    config_paths.push(PathBuf::from(".cookdocs.yaml"));
    config_paths.push(PathBuf::from(".cookdocs.yml"));

    let mut found_path = None;
    for path in config_paths {
        if path.exists() && path.is_file() {
            found_path = Some(path);
            break;
        }
    }

    // 3. Merge config file if found
    if let Some(config_file) = found_path.and_then(|path| Figment::new().merge(Yaml::file(path)).extract::<ConfigFile>().ok()) {
        if let Some(val) = config_file.dry_run { config.dry_run = val; }
        if let Some(val) = config_file.jsonify { config.jsonify = val; }
        if let Some(val) = config_file.ignore_file { config.ignore_file = val; }
        if let Some(val) = config_file.recipe_search_root { config.recipe_search_root = val; }
        if let Some(val) = config_file.log_level { config.log_level = val; }
        if let Some(val) = config_file.template_files { config.template_files = val; }
        if let Some(val) = config_file.word_wrap { config.word_wrap = val; }
        if let Some(val) = config_file.output_dir { config.output_dir = val; }
    }

    // 4. Merge environment variables
    if let Ok(val) = std::env::var("COOK_DOCS_DRY_RUN") {
        config.dry_run = val.parse().unwrap_or(config.dry_run);
    }
    if let Ok(val) = std::env::var("COOK_DOCS_JSONIFY") {
        config.jsonify = val.parse().unwrap_or(config.jsonify);
    }
    if let Ok(val) = std::env::var("COOK_DOCS_IGNORE_FILE") {
        config.ignore_file = val;
    }
    if let Ok(val) = std::env::var("COOK_DOCS_RECIPE_SEARCH_ROOT") {
        config.recipe_search_root = val;
    }
    if let Ok(val) = std::env::var("COOK_DOCS_LOG_LEVEL") {
        config.log_level = val;
    }
    if let Ok(val) = std::env::var("COOK_DOCS_TEMPLATE_FILES") {
        config.template_files = val.split(',').map(|s| s.trim().to_string()).collect();
    }
    if let Ok(val) = std::env::var("COOK_DOCS_WORD_WRAP") {
        config.word_wrap = val.parse().unwrap_or(config.word_wrap);
    }
    if let Ok(val) = std::env::var("COOK_DOCS_OUTPUT_DIR") {
        config.output_dir = val;
    }

    // 5. Merge CLI arguments (using clap)
    let matches = Command::new("cook-docs")
        .arg(Arg::new("dry-run")
            .short('d')
            .long("dry-run")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("jsonify")
            .short('j')
            .long("jsonify")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("ignore-file")
            .short('i')
            .long("ignore-file"))
        .arg(Arg::new("recipe-search-root")
            .short('c')
            .long("recipe-search-root"))
        .arg(Arg::new("log-level")
            .short('l')
            .long("log-level"))
        .arg(Arg::new("template-files")
            .short('t')
            .long("template-files"))
        .arg(Arg::new("word-wrap")
            .short('w')
            .long("word-wrap"))
        .arg(Arg::new("output-dir")
            .short('o')
            .long("output-dir"))
        .try_get_matches_from(args)
        .map_err(|e| e.to_string())?;

    if matches.value_source("dry-run") == Some(clap::parser::ValueSource::CommandLine) {
        config.dry_run = matches.get_flag("dry-run");
    }
    if matches.value_source("jsonify") == Some(clap::parser::ValueSource::CommandLine) {
        config.jsonify = matches.get_flag("jsonify");
    }
    if let Some(val) = matches.get_one::<String>("ignore-file") {
        config.ignore_file = val.clone();
    }
    if let Some(val) = matches.get_one::<String>("recipe-search-root") {
        config.recipe_search_root = val.clone();
    }
    if let Some(val) = matches.get_one::<String>("log-level") {
        config.log_level = val.clone();
    }
    if let Some(val) = matches.get_one::<String>("template-files") {
        config.template_files = val.split(',').map(|s| s.trim().to_string()).collect();
    }
    if let Some(parsed) = matches.get_one::<String>("word-wrap").and_then(|val| val.parse::<usize>().ok()) {
        config.word_wrap = parsed;
    }
    if let Some(val) = matches.get_one::<String>("output-dir") {
        config.output_dir = val.clone();
    }

    Ok(config)
}

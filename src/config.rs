use crate::types::Config;
use clap::{Arg, ArgAction, Command};
use clap_complete::Shell;
use figment::{
    Figment,
    providers::{Format, Json, Toml, Yaml},
};
use serde::Deserialize;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum CliAction {
    Run(Config),
    Completions(Shell),
    Man,
    Doctor {
        json: bool,
        check_recipes: bool,
        strict: bool,
        only: Vec<crate::doctor::RecipeCheckKind>,
        config_path: Option<PathBuf>,
        config_result: Result<Config, String>,
    },
    SelfUpdate {
        check: bool,
        yes: bool,
        version: Option<String>,
    },
    Watch {
        config: Config,
        debounce_ms: u64,
    },
}

fn debounce_arg() -> Arg {
    Arg::new("debounce")
        .long("debounce")
        .value_name("MS")
        .value_parser(clap::value_parser!(u64))
        .default_value("300")
        .help("debounce delay in milliseconds for watch mode (default 300)")
}

/// Build the clap CLI command definition for `cook-docs`.
pub fn build_cli() -> Command {
    Command::new("cook-docs")
        .about("cook-docs automatically generates markdown documentation for cook recipes from template files")
        .version(env!("PROJECT_VERSION"))
        .disable_version_flag(true)
        .arg(Arg::new("version")
            .short('v')
            .long("version")
            .action(ArgAction::Version)
            .help("version for cook-docs"))
        .arg(Arg::new("config")
            .short('C')
            .long("config")
            .global(true)
            .help("path to configuration file (TOML, YAML, or JSON)"))
        .arg(Arg::new("dry-run")
            .short('d')
            .long("dry-run")
            .global(true)
            .action(ArgAction::SetTrue)
            .help("don't actually render any markdown files just print to stdout passed"))
        .arg(Arg::new("jsonify")
            .short('j')
            .long("jsonify")
            .global(true)
            .action(ArgAction::SetTrue)
            .help("parse the recipe and display it in json format"))
        .arg(Arg::new("ignore-file")
            .short('i')
            .long("ignore-file")
            .global(true)
            .help("filename to use as an ignore file to exclude recipe directories (default \".cookdocsignore\")"))
        .arg(Arg::new("recipe-search-root")
            .short('c')
            .long("recipe-search-root")
            .global(true)
            .help("directory to search recursively within for recipes (default \".\")"))
        .arg(Arg::new("log-level")
            .short('l')
            .long("log-level")
            .global(true)
            .help("level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default \"info\")"))
        .arg(Arg::new("template-files")
            .short('t')
            .long("template-files")
            .global(true)
            .help("gotemplate file paths relative to each recipe directory from which documentation will be generated (default \"recipe.md.gotmpl\")"))
        .arg(Arg::new("word-wrap")
            .short('w')
            .long("word-wrap")
            .global(true)
            .help("word wrap line length for recipe steps section (default 120)"))
        .arg(Arg::new("output-dir")
            .short('o')
            .long("output-dir")
            .global(true)
            .help("custom output directory to write generated markdown documentation"))
        .arg(Arg::new("assets-dir")
            .long("assets-dir")
            .global(true)
            .help("optional directory to copy companion recipe images into (slug-named)"))
        .arg(Arg::new("image-ext")
            .long("image-ext")
            .global(true)
            .help("optional image extension override for Info.ImageSlug (e.g. webp)"))
        .arg(Arg::new("emoji-file")
            .long("emoji-file")
            .global(true)
            .help("optional YAML/TOML/JSON file mapping ingredients and cookware to emojis"))
        .arg(Arg::new("weights-file")
            .long("weights-file")
            .global(true)
            .help("optional YAML/TOML/JSON ingredient density file for toGrams volume-to-weight conversion"))
        .arg(Arg::new("link-dirs")
            .long("link-dirs")
            .global(true)
            .help("comma-separated recipe subdirectories eligible for linkRecipe cross-linking"))
        .arg(Arg::new("link-in-steps")
            .long("link-in-steps")
            .global(true)
            .action(ArgAction::SetTrue)
            .help("cross-link matching @ingredient references inside step directions"))
        .arg(Arg::new("required-metadata")
            .long("required-metadata")
            .global(true)
            .help("comma-separated metadata keys required by doctor --check-recipes (e.g. source,serves)"))
        .arg(Arg::new("ignored-ingredients-file")
            .long("ignored-ingredients-file")
            .global(true)
            .help("optional YAML/TOML/JSON list of ingredients that should not be tagged with '@' (checked by doctor --check-recipes)"))
        .arg(Arg::new("merge")
            .long("merge")
            .global(true)
            .action(ArgAction::SetTrue)
            .help("merge generated content into existing Markdown files without clobbering manual edits"))
        .arg(Arg::new("force")
            .long("force")
            .global(true)
            .action(ArgAction::SetTrue)
            .help("force full overwrite of existing Markdown files even when --merge / merge is enabled"))
        .arg(Arg::new("files")
            .value_name("FILES")
            .num_args(0..)
            .help("optional recipe files or subdirectories to render (mirrored relative to --recipe-search-root)"))
        .arg(Arg::new("watch")
            .long("watch")
            .action(ArgAction::SetTrue)
            .help("watch recipes and templates and re-render automatically on changes"))
        .arg(debounce_arg())
        .subcommand(
            Command::new("watch")
                .about("Watch recipes and templates and re-render automatically on changes")
                .arg(debounce_arg()),
        )
        .subcommand(
            Command::new("completions")
                .about("Generate shell autocompletions script")
                .arg(
                    Arg::new("shell")
                        .required(true)
                        .value_name("SHELL")
                        .value_parser(clap::value_parser!(Shell))
                        .help("Shell to generate completions for"),
                ),
        )
        .subcommand(Command::new("man").about("Generate UNIX roff man page"))
        .subcommand(
            Command::new("doctor")
                .alias("doc")
                .about("Validate configuration, inputs, templates, and output permissions")
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("Output doctor report in JSON format"),
                )
                .arg(
                    Arg::new("check-recipes")
                        .long("check-recipes")
                        .action(ArgAction::SetTrue)
                        .help("Validate recipe syntax, companion images, required metadata, emoji mappings, and ignored ingredients"),
                )
                .arg(
                    Arg::new("only")
                        .long("only")
                        .value_name("CATEGORIES")
                        .action(ArgAction::Append)
                        .help("Comma-separated recipe check categories to run (syntax,images,metadata,emojis,ignored)"),
                )
                .arg(
                    Arg::new("strict")
                        .long("strict")
                        .action(ArgAction::SetTrue)
                        .help("Treat warnings as unhealthy and exit non-zero"),
                )
                .arg(
                    Arg::new("doctor-files")
                        .value_name("FILES")
                        .num_args(0..)
                        .help("Optional recipe files or subdirectories to validate"),
                ),
        )
        .subcommand(
            Command::new("self-update")
                .alias("update")
                .about("Check for and install the latest cook-docs release from GitHub")
                .arg(
                    Arg::new("check")
                        .long("check")
                        .action(ArgAction::SetTrue)
                        .help("Check for available updates without installing"),
                )
                .arg(
                    Arg::new("yes")
                        .short('y')
                        .long("yes")
                        .action(ArgAction::SetTrue)
                        .help("Skip confirmation prompt before installing"),
                )
                .arg(
                    Arg::new("version")
                        .long("version")
                        .value_name("VERSION")
                        .help("Target a specific release version or tag (e.g. 0.9.1 or v0.9.1)"),
                ),
        )
}

/// Write static shell completions for `shell` to `writer`.
pub fn generate_completions(shell: Shell, writer: &mut dyn Write) {
    let mut cmd = build_cli();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, bin_name, writer);
}

/// Write a UNIX roff man page for `cook-docs` to `writer`.
pub fn generate_manpage(writer: &mut dyn Write) -> Result<(), String> {
    let cmd = build_cli();
    clap_mangen::Man::new(cmd)
        .render(writer)
        .map_err(|e| format!("Failed rendering man page: {e}"))
}

#[derive(Deserialize, Debug, Default, PartialEq)]
struct ConfigFile {
    #[serde(alias = "dry-run", alias = "dry_run", alias = "dryRun")]
    pub dry_run: Option<bool>,
    #[serde(alias = "jsonify")]
    pub jsonify: Option<bool>,
    #[serde(alias = "ignore-file", alias = "ignore_file", alias = "ignoreFile")]
    pub ignore_file: Option<String>,
    #[serde(
        alias = "recipe-search-root",
        alias = "recipe_search_root",
        alias = "recipeSearchRoot"
    )]
    pub recipe_search_root: Option<String>,
    #[serde(alias = "log-level", alias = "log_level", alias = "logLevel")]
    pub log_level: Option<String>,
    #[serde(
        alias = "template-files",
        alias = "template_files",
        alias = "templateFiles"
    )]
    pub template_files: Option<Vec<String>>,
    #[serde(alias = "word-wrap", alias = "word_wrap", alias = "wordWrap")]
    pub word_wrap: Option<usize>,
    #[serde(alias = "output-dir", alias = "output_dir", alias = "outputDir")]
    pub output_dir: Option<String>,
    #[serde(alias = "assets-dir", alias = "assets_dir", alias = "assetsDir")]
    pub assets_dir: Option<String>,
    #[serde(alias = "image-ext", alias = "image_ext", alias = "imageExt")]
    pub image_ext: Option<String>,
    #[serde(alias = "emoji-file", alias = "emoji_file", alias = "emojiFile")]
    pub emoji_file: Option<String>,
    #[serde(alias = "emoji")]
    pub emoji: Option<serde_json::Value>,
    #[serde(alias = "weights-file", alias = "weights_file", alias = "weightsFile")]
    pub weights_file: Option<String>,
    #[serde(alias = "link-dirs", alias = "link_dirs", alias = "linkDirs")]
    pub link_dirs: Option<Vec<String>>,
    #[serde(
        alias = "link-in-steps",
        alias = "link_in_steps",
        alias = "linkInSteps"
    )]
    pub link_in_steps: Option<bool>,
    #[serde(
        alias = "category-tags",
        alias = "category_tags",
        alias = "categoryTags",
        default,
        deserialize_with = "crate::types::deserialize_opt_category_tags"
    )]
    pub category_tags: Option<std::collections::BTreeMap<String, Vec<String>>>,
    #[serde(
        alias = "required-metadata",
        alias = "required_metadata",
        alias = "requiredMetadata"
    )]
    pub required_metadata: Option<Vec<String>>,
    #[serde(
        alias = "ignored-ingredients-file",
        alias = "ignored_ingredients_file",
        alias = "ignoredIngredientsFile"
    )]
    pub ignored_ingredients_file: Option<String>,
    #[serde(
        alias = "ignored-ingredients",
        alias = "ignored_ingredients",
        alias = "ignoredIngredients"
    )]
    pub ignored_ingredients: Option<Vec<String>>,
    #[serde(alias = "merge")]
    pub merge: Option<bool>,
}

fn parse_csv_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn load_config(args: &[&str]) -> Result<Config, String> {
    match load_action(args)? {
        CliAction::Run(cfg) => Ok(cfg),
        CliAction::Completions(_)
        | CliAction::Man
        | CliAction::Doctor { .. }
        | CliAction::SelfUpdate { .. }
        | CliAction::Watch { .. } => Err("Subcommand cannot be used as config".to_string()),
    }
}

fn default_config_paths(home_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut config_paths = Vec::new();
    let push_dir = |paths: &mut Vec<PathBuf>, dir: &Path| {
        for ext in ["toml", "yaml", "yml", "json"] {
            paths.push(dir.join(format!(".cookdocs.{ext}")));
        }
    };

    push_dir(&mut config_paths, Path::new("/etc/cook-docs"));

    if let Some(home) = home_dir
        && !home.as_os_str().is_empty()
    {
        push_dir(&mut config_paths, &home.join(".config"));
    }

    push_dir(&mut config_paths, Path::new("."));
    config_paths
}

/// Parse CLI args into an actionable command, loading config files and env vars when running normally.
pub fn load_action(args: &[&str]) -> Result<CliAction, String> {
    let config_paths = default_config_paths(std::env::home_dir().as_deref());
    load_action_with(args, &config_paths, |k| std::env::var(k).ok())
}

fn load_config_file(path: &Path) -> Result<ConfigFile, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());

    let figment = match ext.as_deref() {
        Some("toml") => Figment::new().merge(Toml::file(path)),
        Some("yaml") | Some("yml") => Figment::new().merge(Yaml::file(path)),
        Some("json") => Figment::new().merge(Json::file(path)),
        _ => {
            return Err(format!(
                "Unsupported config file format '{}': expected .toml, .yaml, .yml, or .json",
                path.display()
            ));
        }
    };

    figment
        .extract::<ConfigFile>()
        .map_err(|e| format!("Failed to parse config file '{}': {e}", path.display()))
}

fn load_action_with<F>(
    args: &[&str],
    config_paths: &[PathBuf],
    env_get: F,
) -> Result<CliAction, String>
where
    F: Fn(&str) -> Option<String>,
{
    let matches = build_cli().try_get_matches_from(args).map_err(|e| {
        if e.use_stderr() {
            e.to_string()
        } else {
            let _ = e.print();
            std::process::exit(0);
        }
    })?;

    if let Some(("completions", sub_matches)) = matches.subcommand() {
        let shell = *sub_matches
            .get_one::<Shell>("shell")
            .ok_or_else(|| "Missing shell argument".to_string())?;
        return Ok(CliAction::Completions(shell));
    }
    if let Some(("man", _)) = matches.subcommand() {
        return Ok(CliAction::Man);
    }
    if let Some(("doctor", sub_matches)) = matches.subcommand() {
        let json = sub_matches.get_flag("json");
        let strict = sub_matches.get_flag("strict");
        let mut only = Vec::new();
        if let Some(vals) = sub_matches.get_many::<String>("only") {
            for raw_arg in vals {
                let parts = parse_csv_list(raw_arg);
                if parts.is_empty() {
                    return Err(
                        "Empty --only value: expected one or more of syntax, images, metadata, emojis, ignored"
                            .to_string(),
                    );
                }
                for part in parts {
                    let kind = crate::doctor::RecipeCheckKind::parse(&part)?;
                    if !only.contains(&kind) {
                        only.push(kind);
                    }
                }
            }
        }
        let check_recipes = sub_matches.get_flag("check-recipes") || !only.is_empty();
        let (config_path, mut config_result) =
            resolve_config_with_source(&matches, config_paths, env_get);
        if let Ok(ref mut cfg) = config_result
            && let Some(files) = sub_matches.get_many::<String>("doctor-files")
        {
            cfg.recipe_files = files.cloned().collect();
        }
        return Ok(CliAction::Doctor {
            json,
            check_recipes,
            strict,
            only,
            config_path,
            config_result,
        });
    }
    if let Some(("self-update", sub_matches)) = matches.subcommand() {
        let check = sub_matches.get_flag("check");
        let yes = sub_matches.get_flag("yes");
        let version = sub_matches.get_one::<String>("version").cloned();
        return Ok(CliAction::SelfUpdate {
            check,
            yes,
            version,
        });
    }
    if let Some(("watch", sub_matches)) = matches.subcommand() {
        let debounce_ms = if sub_matches.value_source("debounce")
            == Some(clap::parser::ValueSource::CommandLine)
        {
            *sub_matches
                .get_one::<u64>("debounce")
                .unwrap_or(&crate::watch::DEFAULT_DEBOUNCE_MS)
        } else {
            *matches
                .get_one::<u64>("debounce")
                .unwrap_or(&crate::watch::DEFAULT_DEBOUNCE_MS)
        };
        let (_, config_result) = resolve_config_with_source(&matches, config_paths, env_get);
        return Ok(CliAction::Watch {
            config: config_result?,
            debounce_ms,
        });
    }

    let (_, config_result) = resolve_config_with_source(&matches, config_paths, env_get);
    let config = config_result?;
    if matches.get_flag("watch") {
        let debounce_ms = *matches
            .get_one::<u64>("debounce")
            .unwrap_or(&crate::watch::DEFAULT_DEBOUNCE_MS);
        return Ok(CliAction::Watch {
            config,
            debounce_ms,
        });
    }

    Ok(CliAction::Run(config))
}

#[cfg(test)]
fn load_config_with<F>(
    args: &[&str],
    config_paths: &[PathBuf],
    env_get: F,
) -> Result<Config, String>
where
    F: Fn(&str) -> Option<String>,
{
    match load_action_with(args, config_paths, env_get)? {
        CliAction::Run(cfg) => Ok(cfg),
        CliAction::Completions(_)
        | CliAction::Man
        | CliAction::Doctor { .. }
        | CliAction::SelfUpdate { .. }
        | CliAction::Watch { .. } => Err("Subcommand cannot be used as config".to_string()),
    }
}

fn resolve_config_with_source<F>(
    matches: &clap::ArgMatches,
    config_paths: &[PathBuf],
    env_get: F,
) -> (Option<PathBuf>, Result<Config, String>)
where
    F: Fn(&str) -> Option<String>,
{
    // 1. Set default configuration values
    let mut config = Config::default();

    // 2. Resolve the config file: explicit -C/--config or COOK_DOCS_CONFIG, else default discovery
    let explicit_config = matches
        .get_one::<String>("config")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            env_get("COOK_DOCS_CONFIG")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        });

    let (resolved_path, loaded_config_file) = if let Some(explicit_path) = explicit_config {
        let path = PathBuf::from(&explicit_path);
        if !path.exists() {
            return (
                Some(path.clone()),
                Err(format!("Configuration file '{}' not found", path.display())),
            );
        }
        if !path.is_file() {
            return (
                Some(path.clone()),
                Err(format!(
                    "Configuration path '{}' is not a regular file",
                    path.display()
                )),
            );
        }
        match load_config_file(&path) {
            Ok(cfg) => (Some(path), Some(cfg)),
            Err(e) => return (Some(path), Err(e)),
        }
    } else {
        let found_path = config_paths
            .iter()
            .find(|path| path.exists() && path.is_file())
            .cloned();
        match found_path {
            Some(path) => match load_config_file(&path) {
                Ok(cfg) => (Some(path), Some(cfg)),
                Err(e) => return (Some(path), Err(e)),
            },
            None => (None, None),
        }
    };

    // 3. Merge config file if found
    if let Some(config_file) = loaded_config_file {
        if let Some(val) = config_file.dry_run {
            config.dry_run = val;
        }
        if let Some(val) = config_file.jsonify {
            config.jsonify = val;
        }
        if let Some(val) = config_file.ignore_file {
            config.ignore_file = val;
        }
        if let Some(val) = config_file.recipe_search_root {
            config.recipe_search_root = val;
        }
        if let Some(val) = config_file.log_level {
            config.log_level = val;
        }
        if let Some(val) = config_file.template_files {
            config.template_files = val;
        }
        if let Some(val) = config_file.word_wrap {
            config.word_wrap = val;
        }
        if let Some(val) = config_file.output_dir {
            config.output_dir = val;
        }
        if let Some(val) = config_file.assets_dir {
            config.assets_dir = val;
        }
        if let Some(val) = config_file.image_ext {
            config.image_ext = val;
        }
        if let Some(val) = config_file.emoji_file {
            config.emoji_file = val;
        }
        if let Some(val) = config_file.emoji {
            config.emoji = Some(val);
        }
        if let Some(val) = config_file.weights_file {
            config.weights_file = val;
        }
        if let Some(val) = config_file.link_dirs {
            config.link_dirs = val;
        }
        if let Some(val) = config_file.link_in_steps {
            config.link_in_steps = val;
        }
        if let Some(val) = config_file.category_tags {
            config.category_tags = val;
        }
        if let Some(val) = config_file.required_metadata {
            config.required_metadata = val;
        }
        if let Some(val) = config_file.ignored_ingredients_file {
            config.ignored_ingredients_file = val;
        }
        if let Some(val) = config_file.ignored_ingredients {
            config.ignored_ingredients = val;
        }
        if let Some(val) = config_file.merge {
            config.merge = val;
        }
    }

    // 4. Merge environment variables
    if let Some(val) = env_get("COOK_DOCS_DRY_RUN") {
        config.dry_run = val.parse().unwrap_or(config.dry_run);
    }
    if let Some(val) = env_get("COOK_DOCS_JSONIFY") {
        config.jsonify = val.parse().unwrap_or(config.jsonify);
    }
    if let Some(val) = env_get("COOK_DOCS_IGNORE_FILE") {
        config.ignore_file = val;
    }
    if let Some(val) = env_get("COOK_DOCS_RECIPE_SEARCH_ROOT") {
        config.recipe_search_root = val;
    }
    if let Some(val) = env_get("COOK_DOCS_LOG_LEVEL") {
        config.log_level = val;
    }
    if let Some(val) = env_get("COOK_DOCS_TEMPLATE_FILES") {
        config.template_files = val.split(',').map(|s| s.trim().to_string()).collect();
    }
    if let Some(val) = env_get("COOK_DOCS_WORD_WRAP") {
        config.word_wrap = val.parse().unwrap_or(config.word_wrap);
    }
    if let Some(val) = env_get("COOK_DOCS_OUTPUT_DIR") {
        config.output_dir = val;
    }
    if let Some(val) = env_get("COOK_DOCS_ASSETS_DIR") {
        config.assets_dir = val;
    }
    if let Some(val) = env_get("COOK_DOCS_IMAGE_EXT") {
        config.image_ext = val;
    }
    if let Some(val) = env_get("COOK_DOCS_EMOJI_FILE") {
        config.emoji_file = val;
    }
    if let Some(val) = env_get("COOK_DOCS_WEIGHTS_FILE") {
        config.weights_file = val;
    }
    if let Some(val) = env_get("COOK_DOCS_LINK_DIRS") {
        config.link_dirs = parse_csv_list(&val);
    }
    if let Some(val) = env_get("COOK_DOCS_LINK_IN_STEPS") {
        config.link_in_steps = val.parse().unwrap_or(config.link_in_steps);
    }
    if let Some(val) = env_get("COOK_DOCS_REQUIRED_METADATA") {
        config.required_metadata = parse_csv_list(&val);
    }
    if let Some(val) = env_get("COOK_DOCS_IGNORED_INGREDIENTS_FILE") {
        config.ignored_ingredients_file = val;
    }
    if let Some(val) = env_get("COOK_DOCS_IGNORED_INGREDIENTS") {
        config.ignored_ingredients = parse_csv_list(&val);
    }
    if let Some(val) = env_get("COOK_DOCS_MERGE") {
        config.merge = val.parse().unwrap_or(config.merge);
    }
    if let Some(val) = env_get("COOK_DOCS_FORCE") {
        config.force = val.parse().unwrap_or(config.force);
    }

    // 5. Merge CLI arguments
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
    if let Some(parsed) = matches
        .get_one::<String>("word-wrap")
        .and_then(|val| val.parse::<usize>().ok())
    {
        config.word_wrap = parsed;
    }
    if let Some(val) = matches.get_one::<String>("output-dir") {
        config.output_dir = val.clone();
    }
    if let Some(val) = matches.get_one::<String>("assets-dir") {
        config.assets_dir = val.clone();
    }
    if let Some(val) = matches.get_one::<String>("image-ext") {
        config.image_ext = val.clone();
    }
    if let Some(val) = matches.get_one::<String>("emoji-file") {
        config.emoji_file = val.clone();
    }
    if let Some(val) = matches.get_one::<String>("weights-file") {
        config.weights_file = val.clone();
    }
    if let Some(val) = matches.get_one::<String>("link-dirs") {
        config.link_dirs = parse_csv_list(val);
    }
    if matches.value_source("link-in-steps") == Some(clap::parser::ValueSource::CommandLine) {
        config.link_in_steps = matches.get_flag("link-in-steps");
    }
    if let Some(val) = matches.get_one::<String>("required-metadata") {
        config.required_metadata = parse_csv_list(val);
    }
    if let Some(val) = matches.get_one::<String>("ignored-ingredients-file") {
        config.ignored_ingredients_file = val.clone();
    }
    if matches.value_source("merge") == Some(clap::parser::ValueSource::CommandLine) {
        config.merge = matches.get_flag("merge");
    }
    if matches.value_source("force") == Some(clap::parser::ValueSource::CommandLine) {
        config.force = matches.get_flag("force");
    }
    if let Some(files) = matches.get_many::<String>("files") {
        config.recipe_files = files.cloned().collect();
    }
    config.image_ext = config
        .image_ext
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase();

    (resolved_path, Ok(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn test_load_config_with_yaml_file_and_precedence() {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("cook_docs_cfg_{}_{}", std::process::id(), id));
        fs::create_dir_all(&dir).unwrap();
        let cfg_file = dir.join(".cookdocs.yaml");
        fs::write(
            &cfg_file,
            "\
dryRun: true
jsonify: true
ignoreFile: .myignore
recipeSearchRoot: recipes
logLevel: debug
templateFiles:
  - custom.gotmpl
wordWrap: 90
outputDir: build
",
        )
        .unwrap();

        let empty_env = |_: &str| None;
        let cfg =
            load_config_with(&["cook-docs"], std::slice::from_ref(&cfg_file), empty_env).unwrap();
        assert!(cfg.dry_run);
        assert!(cfg.jsonify);
        assert_eq!(cfg.ignore_file, ".myignore");
        assert_eq!(cfg.recipe_search_root, "recipes");
        assert_eq!(cfg.log_level, "debug");
        assert_eq!(cfg.template_files, vec!["custom.gotmpl"]);
        assert_eq!(cfg.word_wrap, 90);
        assert_eq!(cfg.output_dir, "build");

        let mut env_map = HashMap::new();
        env_map.insert("COOK_DOCS_DRY_RUN", "false");
        env_map.insert("COOK_DOCS_JSONIFY", "notabool");
        env_map.insert("COOK_DOCS_IGNORE_FILE", ".envignore");
        env_map.insert("COOK_DOCS_RECIPE_SEARCH_ROOT", "env_root");
        env_map.insert("COOK_DOCS_LOG_LEVEL", "warn");
        env_map.insert("COOK_DOCS_TEMPLATE_FILES", "a.gotmpl, b.gotmpl");
        env_map.insert("COOK_DOCS_WORD_WRAP", "72");
        env_map.insert("COOK_DOCS_OUTPUT_DIR", "env_out");

        let cfg2 = load_config_with(&["cook-docs", "-w", "notanint"], &[cfg_file], |k| {
            env_map.get(k).map(|s| s.to_string())
        })
        .unwrap();
        assert!(!cfg2.dry_run);
        assert!(cfg2.jsonify);
        assert_eq!(cfg2.ignore_file, ".envignore");
        assert_eq!(cfg2.recipe_search_root, "env_root");
        assert_eq!(cfg2.log_level, "warn");
        assert_eq!(cfg2.template_files, vec!["a.gotmpl", "b.gotmpl"]);
        assert_eq!(cfg2.word_wrap, 72);
        assert_eq!(cfg2.output_dir, "env_out");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_config_with_invalid_cli_arg_returns_err() {
        let err = load_config_with(&["cook-docs", "--no-such-flag"], &[], |_| None).unwrap_err();
        assert!(err.contains("unexpected argument"));
    }

    #[test]
    fn test_load_action_completions_and_generate() {
        let shells = [
            ("bash", Shell::Bash),
            ("zsh", Shell::Zsh),
            ("fish", Shell::Fish),
            ("powershell", Shell::PowerShell),
            ("elvish", Shell::Elvish),
        ];
        for (name, expected) in shells {
            let action = load_action_with(&["cook-docs", "completions", name], &[], |_| None)
                .expect("valid shell");
            match action {
                CliAction::Completions(shell) => {
                    assert_eq!(shell, expected);
                    let mut buf = Vec::new();
                    generate_completions(shell, &mut buf);
                    let script = String::from_utf8(buf).unwrap();
                    assert!(script.contains("cook-docs"), "empty script for {name}");
                }
                CliAction::Run(_)
                | CliAction::Man
                | CliAction::Doctor { .. }
                | CliAction::SelfUpdate { .. }
                | CliAction::Watch { .. } => {
                    panic!("expected Completions action for {name}")
                }
            }
        }

        assert!(load_action_with(&["cook-docs", "completions"], &[], |_| None).is_err());
        assert!(load_action_with(&["cook-docs", "completions", "tcsh"], &[], |_| None).is_err());
        assert!(load_config_with(&["cook-docs", "completions", "bash"], &[], |_| None).is_err());
    }

    #[test]
    fn test_load_action_man_and_generate() {
        let action = load_action_with(&["cook-docs", "man"], &[], |_| None).expect("valid man");
        assert!(matches!(action, CliAction::Man));
        let mut buf = Vec::new();
        generate_manpage(&mut buf).unwrap();
        let roff = String::from_utf8(buf).unwrap();
        assert!(roff.contains(".TH cook-docs 1"));
        assert!(roff.contains("OPTIONS"));
        assert!(load_config_with(&["cook-docs", "man"], &[], |_| None).is_err());
        assert!(load_action_with(&["cook-docs", "man", "extra"], &[], |_| None).is_err());
    }

    #[test]
    fn test_load_config_toml_and_json_files() {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("cook_docs_toml_json_{}_{}", std::process::id(), id));
        fs::create_dir_all(&dir).unwrap();

        let toml_file = dir.join(".cookdocs.toml");
        fs::write(
            &toml_file,
            "\
dry_run = true
jsonify = true
ignore_file = \".tomlignore\"
recipe_search_root = \"toml_root\"
log_level = \"debug\"
template_files = [\"t1.gotmpl\", \"t2.gotmpl\"]
word_wrap = 88
output_dir = \"toml_out\"
",
        )
        .unwrap();

        let cfg_toml = load_config_with(&["cook-docs"], &[toml_file], |_| None).unwrap();
        assert!(cfg_toml.dry_run);
        assert!(cfg_toml.jsonify);
        assert_eq!(cfg_toml.ignore_file, ".tomlignore");
        assert_eq!(cfg_toml.recipe_search_root, "toml_root");
        assert_eq!(cfg_toml.log_level, "debug");
        assert_eq!(cfg_toml.template_files, vec!["t1.gotmpl", "t2.gotmpl"]);
        assert_eq!(cfg_toml.word_wrap, 88);
        assert_eq!(cfg_toml.output_dir, "toml_out");

        let json_file = dir.join(".cookdocs.json");
        fs::write(
            &json_file,
            r#"{
  "dryRun": true,
  "ignore-file": ".jsonignore",
  "recipeSearchRoot": "json_root",
  "logLevel": "warn",
  "templateFiles": ["j.gotmpl"],
  "wordWrap": 64,
  "outputDir": "json_out"
}"#,
        )
        .unwrap();

        let cfg_json = load_config_with(&["cook-docs"], &[json_file], |_| None).unwrap();
        assert!(cfg_json.dry_run);
        assert!(!cfg_json.jsonify);
        assert_eq!(cfg_json.ignore_file, ".jsonignore");
        assert_eq!(cfg_json.recipe_search_root, "json_root");
        assert_eq!(cfg_json.log_level, "warn");
        assert_eq!(cfg_json.template_files, vec!["j.gotmpl"]);
        assert_eq!(cfg_json.word_wrap, 64);
        assert_eq!(cfg_json.output_dir, "json_out");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_explicit_config_flag_and_env_precedence() {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("cook_docs_explicit_{}_{}", std::process::id(), id));
        fs::create_dir_all(&dir).unwrap();

        let default_cfg = dir.join(".cookdocs.yaml");
        fs::write(&default_cfg, "logLevel: error\nwordWrap: 40\n").unwrap();

        let env_cfg = dir.join("env.toml");
        fs::write(&env_cfg, "log_level = \"warn\"\nword_wrap = 60\n").unwrap();

        let cli_cfg = dir.join("cli.json");
        fs::write(
            &cli_cfg,
            r#"{"logLevel": "debug", "wordWrap": 80, "dryRun": true}"#,
        )
        .unwrap();

        // 1. COOK_DOCS_CONFIG overrides default config_paths
        let env_path_str = env_cfg.to_string_lossy().to_string();
        let cfg_env = load_config_with(&["cook-docs"], std::slice::from_ref(&default_cfg), |k| {
            (k == "COOK_DOCS_CONFIG").then(|| env_path_str.clone())
        })
        .unwrap();
        assert_eq!(cfg_env.log_level, "warn");
        assert_eq!(cfg_env.word_wrap, 60);

        // 2. -C / --config overrides COOK_DOCS_CONFIG and default config_paths
        let cli_path_str = cli_cfg.to_string_lossy().to_string();
        let cfg_cli = load_config_with(&["cook-docs", "-C", &cli_path_str], &[default_cfg], |k| {
            (k == "COOK_DOCS_CONFIG").then(|| env_path_str.clone())
        })
        .unwrap();
        assert_eq!(cfg_cli.log_level, "debug");
        assert_eq!(cfg_cli.word_wrap, 80);
        assert!(cfg_cli.dry_run);

        // 3. Other CLI flags override values from --config file
        let cfg_cli_override = load_config_with(
            &["cook-docs", "--config", &cli_path_str, "-w", "100"],
            &[],
            |_| None,
        )
        .unwrap();
        assert_eq!(cfg_cli_override.word_wrap, 100);
        assert_eq!(cfg_cli_override.log_level, "debug");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_explicit_config_error_cases() {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("cook_docs_cfg_err_{}_{}", std::process::id(), id));
        fs::create_dir_all(&dir).unwrap();

        let missing = dir.join("missing.toml");
        let missing_str = missing.to_string_lossy().to_string();
        assert!(load_config_with(&["cook-docs", "-C", &missing_str], &[], |_| None).is_err());
        assert!(
            load_config_with(&["cook-docs"], &[], |k| {
                (k == "COOK_DOCS_CONFIG").then(|| missing_str.clone())
            })
            .is_err()
        );

        let dir_str = dir.to_string_lossy().to_string();
        assert!(load_config_with(&["cook-docs", "--config", &dir_str], &[], |_| None).is_err());

        let bad_ext = dir.join("config.txt");
        fs::write(&bad_ext, "dry_run = true").unwrap();
        let bad_ext_str = bad_ext.to_string_lossy().to_string();
        assert!(load_config_with(&["cook-docs", "-C", &bad_ext_str], &[], |_| None).is_err());

        let bad_toml = dir.join("bad.toml");
        fs::write(&bad_toml, "dry_run = [not valid").unwrap();
        let bad_toml_str = bad_toml.to_string_lossy().to_string();
        assert!(load_config_with(&["cook-docs", "-C", &bad_toml_str], &[], |_| None).is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_example_config_file_validity() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let example_path = manifest_dir.join("assets/config.example.toml");
        assert!(
            example_path.is_file(),
            "assets/config.example.toml must exist"
        );

        // Commented-out example config must parse to all-None ConfigFile
        let raw_cfg = load_config_file(&example_path)
            .expect("assets/config.example.toml must parse cleanly as-is");
        assert_eq!(raw_cfg, ConfigFile::default());

        // Uncommenting all setting lines must parse into populated defaults
        let content = fs::read_to_string(&example_path).unwrap();
        let uncommented: String = content
            .lines()
            .map(|line| {
                let trimmed = line.trim_start();
                if let Some(rest) = trimmed.strip_prefix("# ")
                    && (rest.starts_with("dry_run =")
                        || rest.starts_with("jsonify =")
                        || rest.starts_with("ignore_file =")
                        || rest.starts_with("recipe_search_root =")
                        || rest.starts_with("log_level =")
                        || rest.starts_with("template_files =")
                        || rest.starts_with("word_wrap =")
                        || rest.starts_with("output_dir =")
                        || rest.starts_with("assets_dir =")
                        || rest.starts_with("image_ext =")
                        || rest.starts_with("emoji_file =")
                        || rest.starts_with("weights_file =")
                        || rest.starts_with("link_dirs =")
                        || rest.starts_with("link_in_steps =")
                        || rest.starts_with("required_metadata =")
                        || rest.starts_with("ignored_ingredients_file =")
                        || rest.starts_with("ignored_ingredients =")
                        || rest.starts_with("merge ="))
                {
                    return rest.to_string();
                }
                line.to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");

        let parsed: ConfigFile = Figment::new()
            .merge(Toml::string(&uncommented))
            .extract()
            .expect("uncommented assets/config.example.toml must parse");
        assert_eq!(parsed.dry_run, Some(false));
        assert_eq!(parsed.jsonify, Some(false));
        assert_eq!(parsed.ignore_file.as_deref(), Some(".cookdocsignore"));
        assert_eq!(parsed.recipe_search_root.as_deref(), Some("."));
        assert_eq!(parsed.log_level.as_deref(), Some("info"));
        assert_eq!(
            parsed.template_files.as_deref(),
            Some(&["recipe.md.gotmpl".to_string()][..])
        );
        assert_eq!(parsed.word_wrap, Some(120));
        assert_eq!(parsed.output_dir.as_deref(), Some(""));
        assert_eq!(parsed.assets_dir.as_deref(), Some(""));
        assert_eq!(parsed.image_ext.as_deref(), Some(""));
        assert_eq!(parsed.emoji_file.as_deref(), Some(""));
        assert_eq!(parsed.weights_file.as_deref(), Some(""));
        assert_eq!(parsed.link_dirs.as_deref(), Some(&[][..]));
        assert_eq!(parsed.link_in_steps, Some(false));
        assert_eq!(parsed.required_metadata.as_deref(), Some(&[][..]));
        assert_eq!(parsed.ignored_ingredients_file.as_deref(), Some(""));
        assert_eq!(parsed.ignored_ingredients.as_deref(), Some(&[][..]));
        assert_eq!(parsed.merge, Some(false));
    }

    #[test]
    fn test_load_action_doctor_and_alias() {
        for cmd in ["doctor", "doc"] {
            let action = load_action_with(&["cook-docs", cmd], &[], |_| None).unwrap();
            match action {
                CliAction::Doctor {
                    json,
                    check_recipes,
                    strict,
                    only,
                    config_path,
                    config_result,
                } => {
                    assert!(!json);
                    assert!(!check_recipes);
                    assert!(!strict);
                    assert!(only.is_empty());
                    assert!(config_path.is_none());
                    assert!(config_result.is_ok());
                }
                _ => panic!("expected Doctor action for {cmd}"),
            }
        }

        let action_json = load_action_with(
            &[
                "cook-docs",
                "doctor",
                "--json",
                "--check-recipes",
                "--required-metadata",
                "source,serves",
                "recipes/A.cook",
            ],
            &[],
            |_| None,
        )
        .unwrap();
        match action_json {
            CliAction::Doctor {
                json,
                check_recipes,
                config_result,
                ..
            } => {
                assert!(json);
                assert!(check_recipes);
                let cfg = config_result.unwrap();
                assert_eq!(cfg.required_metadata, vec!["source", "serves"]);
                assert_eq!(cfg.recipe_files, vec!["recipes/A.cook"]);
            }
            _ => panic!("expected Doctor action"),
        }
        assert!(load_config_with(&["cook-docs", "doctor"], &[], |_| None).is_err());
    }

    #[test]
    fn test_load_action_self_update_and_alias() {
        let default_action =
            load_action_with(&["cook-docs", "self-update"], &[], |_| None).unwrap();
        assert!(matches!(
            default_action,
            CliAction::SelfUpdate {
                check: false,
                yes: false,
                version: None
            }
        ));

        let flags_action = load_action_with(
            &[
                "cook-docs",
                "self-update",
                "--check",
                "-y",
                "--version",
                "0.9.1",
            ],
            &[],
            |_| None,
        )
        .unwrap();
        assert!(matches!(
            flags_action,
            CliAction::SelfUpdate {
                check: true,
                yes: true,
                version: Some(ref v)
            } if v == "0.9.1"
        ));

        let alias_action =
            load_action_with(&["cook-docs", "update", "--yes"], &[], |_| None).unwrap();
        assert!(matches!(
            alias_action,
            CliAction::SelfUpdate {
                check: false,
                yes: true,
                version: None
            }
        ));

        assert!(load_config_with(&["cook-docs", "self-update"], &[], |_| None).is_err());
    }

    #[test]
    fn test_cli_version_uses_project_version() {
        assert_eq!(build_cli().get_version(), Some(env!("PROJECT_VERSION")));
    }

    #[test]
    fn test_load_action_watch_flag_and_subcommand() {
        // 1. --watch flag on root command
        let flag_action = load_action_with(&["cook-docs", "--watch"], &[], |_| None).unwrap();
        match flag_action {
            CliAction::Watch {
                config,
                debounce_ms,
            } => {
                assert_eq!(debounce_ms, crate::watch::DEFAULT_DEBOUNCE_MS);
                assert!(!config.dry_run);
            }
            other => panic!("expected Watch action for --watch, got {other:?}"),
        }

        // 2. watch subcommand with --debounce and shared options
        let sub_action = load_action_with(
            &[
                "cook-docs",
                "watch",
                "--debounce",
                "150",
                "--dry-run",
                "-c",
                "recipes",
            ],
            &[],
            |_| None,
        )
        .unwrap();
        match sub_action {
            CliAction::Watch {
                config,
                debounce_ms,
            } => {
                assert_eq!(debounce_ms, 150);
                assert!(config.dry_run);
                assert_eq!(config.recipe_search_root, "recipes");
            }
            other => panic!("expected Watch action for watch subcommand, got {other:?}"),
        }

        // 3. --watch with --debounce on root command
        let root_debounce =
            load_action_with(&["cook-docs", "--watch", "--debounce", "500"], &[], |_| {
                None
            })
            .unwrap();
        assert!(matches!(
            root_debounce,
            CliAction::Watch {
                debounce_ms: 500,
                ..
            }
        ));

        // 4. Invalid --debounce rejected
        assert!(
            load_action_with(&["cook-docs", "watch", "--debounce", "abc"], &[], |_| None).is_err()
        );

        // 5. watch subcommand rejected by load_config_with
        assert!(load_config_with(&["cook-docs", "watch"], &[], |_| None).is_err());
    }

    #[test]
    fn test_load_config_rejects_subcommands_and_manpage_write_error() {
        assert!(load_config(&["cook-docs", "man"]).is_err());
        assert!(load_config(&["cook-docs", "watch"]).is_err());

        struct FailingWriter;
        impl Write for FailingWriter {
            fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("disk full"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let err = generate_manpage(&mut FailingWriter).unwrap_err();
        assert!(err.contains("Failed rendering man page"));
    }

    #[test]
    fn test_positional_files_assets_dir_and_image_ext_precedence() {
        let mut env = HashMap::new();
        env.insert("COOK_DOCS_ASSETS_DIR".to_string(), "env_assets".to_string());
        env.insert("COOK_DOCS_IMAGE_EXT".to_string(), ".WEBP".to_string());

        let cfg_env = load_config_with(&["cook-docs"], &[], |k| env.get(k).cloned()).unwrap();
        assert_eq!(cfg_env.assets_dir, "env_assets");
        assert_eq!(cfg_env.image_ext, "webp");
        assert!(cfg_env.recipe_files.is_empty());

        let cfg_cli = load_config_with(
            &[
                "cook-docs",
                "-c",
                "cook",
                "-o",
                "docs",
                "--assets-dir",
                "docs/assets/images",
                "--image-ext",
                ".PNG",
                "cook/breads/Bagels.cook",
                "cook/soups/Chili.cook",
            ],
            &[],
            |k| env.get(k).cloned(),
        )
        .unwrap();
        assert_eq!(cfg_cli.assets_dir, "docs/assets/images");
        assert_eq!(cfg_cli.image_ext, "png");
        assert_eq!(
            cfg_cli.recipe_files,
            vec!["cook/breads/Bagels.cook", "cook/soups/Chili.cook"]
        );

        // Subcommands still take precedence over positional files
        assert!(matches!(
            load_action_with(&["cook-docs", "doctor"], &[], |_| None).unwrap(),
            CliAction::Doctor { .. }
        ));
        assert!(matches!(
            load_action_with(&["cook-docs", "watch"], &[], |_| None).unwrap(),
            CliAction::Watch { .. }
        ));
    }

    #[test]
    fn test_batch_c_config_env_and_cli_precedence() {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("cook_docs_batch_c_{}_{}", std::process::id(), id));
        fs::create_dir_all(&dir).unwrap();
        let cfg_file = dir.join(".cookdocs.yaml");
        fs::write(
            &cfg_file,
            "\
emojiFile: file_emoji.yaml
weightsFile: file_weights.yaml
linkDirs:
  - sauces
  - ingredients
requiredMetadata:
  - source
emoji:
  ingredients:
    - herb:
        - cilantro
",
        )
        .unwrap();

        let from_file =
            load_config_with(&["cook-docs"], std::slice::from_ref(&cfg_file), |_| None).unwrap();
        assert_eq!(from_file.emoji_file, "file_emoji.yaml");
        assert_eq!(from_file.weights_file, "file_weights.yaml");
        assert_eq!(from_file.link_dirs, vec!["sauces", "ingredients"]);
        assert_eq!(from_file.required_metadata, vec!["source"]);
        assert!(from_file.emoji.is_some());

        let mut env = HashMap::new();
        env.insert("COOK_DOCS_EMOJI_FILE", "env_emoji.yaml");
        env.insert("COOK_DOCS_WEIGHTS_FILE", "env_weights.yaml");
        env.insert("COOK_DOCS_LINK_DIRS", "breads, sauces");
        env.insert("COOK_DOCS_REQUIRED_METADATA", "source, serves");
        let from_env = load_config_with(&["cook-docs"], std::slice::from_ref(&cfg_file), |k| {
            env.get(k).map(|s| s.to_string())
        })
        .unwrap();
        assert_eq!(from_env.emoji_file, "env_emoji.yaml");
        assert_eq!(from_env.weights_file, "env_weights.yaml");
        assert_eq!(from_env.link_dirs, vec!["breads", "sauces"]);
        assert_eq!(from_env.required_metadata, vec!["source", "serves"]);

        let from_cli = load_config_with(
            &[
                "cook-docs",
                "--emoji-file",
                "cli_emoji.toml",
                "--weights-file",
                "cli_weights.toml",
                "--link-dirs",
                "condiments",
                "--required-metadata",
                "title",
            ],
            &[cfg_file],
            |k| env.get(k).map(|s| s.to_string()),
        )
        .unwrap();
        assert_eq!(from_cli.emoji_file, "cli_emoji.toml");
        assert_eq!(from_cli.weights_file, "cli_weights.toml");
        assert_eq!(from_cli.link_dirs, vec!["condiments"]);
        assert_eq!(from_cli.required_metadata, vec!["title"]);

        // Merge & force precedence across config file, env, and CLI
        let merge_cfg = dir.join(".cookdocs.toml");
        fs::write(&merge_cfg, "merge = true\n").unwrap();
        let cfg_m =
            load_config_with(&["cook-docs"], std::slice::from_ref(&merge_cfg), |_| None).unwrap();
        assert!(cfg_m.merge);
        assert!(!cfg_m.force);

        let cfg_f = load_config_with(
            &["cook-docs", "--force"],
            std::slice::from_ref(&merge_cfg),
            |_| None,
        )
        .unwrap();
        assert!(cfg_f.merge);
        assert!(cfg_f.force);

        let cfg_env_mf = load_config_with(&["cook-docs"], &[], |k| match k {
            "COOK_DOCS_MERGE" => Some("true".to_string()),
            "COOK_DOCS_FORCE" => Some("true".to_string()),
            _ => None,
        })
        .unwrap();
        assert!(cfg_env_mf.merge);
        assert!(cfg_env_mf.force);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_category_tags_config_formats() {
        let dir = std::env::temp_dir().join(format!(
            "cook_docs_cat_tags_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();

        let toml_cfg = dir.join(".cookdocs.toml");
        fs::write(
            &toml_cfg,
            "\
[category_tags]
beverages = \"beverage\"
breads = [\"bread\"]
\"desserts/cake\" = [\"dessert\", \"cake\"]
",
        )
        .unwrap();
        let cfg_t =
            load_config_with(&["cook-docs"], std::slice::from_ref(&toml_cfg), |_| None).unwrap();
        assert_eq!(
            cfg_t.category_tags.get("beverages"),
            Some(&vec!["beverage".to_string()])
        );
        assert_eq!(
            cfg_t.category_tags.get("desserts/cake"),
            Some(&vec!["dessert".to_string(), "cake".to_string()])
        );

        let yaml_cfg = dir.join(".cookdocs.yaml");
        fs::write(
            &yaml_cfg,
            "\
categoryTags:
  cookies-and-bars:
    - cookie
    - bar
",
        )
        .unwrap();
        let cfg_y =
            load_config_with(&["cook-docs"], std::slice::from_ref(&yaml_cfg), |_| None).unwrap();
        assert_eq!(
            cfg_y.category_tags.get("cookies-and-bars"),
            Some(&vec!["cookie".to_string(), "bar".to_string()])
        );

        let json_cfg = dir.join(".cookdocs.json");
        fs::write(
            &json_cfg,
            r#"{"category-tags": {"soups": ["soup", "comfort"]}}"#,
        )
        .unwrap();
        let cfg_j =
            load_config_with(&["cook-docs"], std::slice::from_ref(&json_cfg), |_| None).unwrap();
        assert_eq!(
            cfg_j.category_tags.get("soups"),
            Some(&vec!["soup".to_string(), "comfort".to_string()])
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_default_config_paths_uses_home_dir() {
        let with_home = default_config_paths(Some(Path::new("/home/alice")));
        assert_eq!(with_home.len(), 12);
        assert_eq!(with_home[0], PathBuf::from("/etc/cook-docs/.cookdocs.toml"));
        assert_eq!(
            with_home[4],
            PathBuf::from("/home/alice/.config/.cookdocs.toml")
        );
        assert_eq!(
            with_home[7],
            PathBuf::from("/home/alice/.config/.cookdocs.json")
        );
        assert_eq!(with_home[8], PathBuf::from("./.cookdocs.toml"));

        let win_home = default_config_paths(Some(Path::new(r"C:\Users\alice")));
        assert_eq!(
            win_home[4],
            Path::new(r"C:\Users\alice")
                .join(".config")
                .join(".cookdocs.toml")
        );

        let without_home = default_config_paths(None);
        assert_eq!(without_home.len(), 8);
        assert_eq!(without_home[4], PathBuf::from("./.cookdocs.toml"));

        let empty_home = default_config_paths(Some(Path::new("")));
        assert_eq!(empty_home, without_home);
    }

    #[test]
    fn test_doctor_only_strict_and_ignored_ingredients_config() {
        let action = load_action_with(
            &[
                "cook-docs",
                "doctor",
                "--strict",
                "--only",
                "syntax,images",
                "--only",
                "ignored",
                "--ignored-ingredients-file",
                "config/ignored_ingredients.yaml",
            ],
            &[],
            |_| None,
        )
        .unwrap();
        match action {
            CliAction::Doctor {
                json,
                check_recipes,
                strict,
                only,
                config_result,
                ..
            } => {
                assert!(!json);
                assert!(check_recipes);
                assert!(strict);
                assert_eq!(
                    only,
                    vec![
                        crate::doctor::RecipeCheckKind::Syntax,
                        crate::doctor::RecipeCheckKind::Images,
                        crate::doctor::RecipeCheckKind::Ignored,
                    ]
                );
                let cfg = config_result.unwrap();
                assert_eq!(
                    cfg.ignored_ingredients_file,
                    "config/ignored_ingredients.yaml"
                );
            }
            other => panic!("expected Doctor action, got {other:?}"),
        }

        // Invalid --only category fails with helpful error
        let err = load_action_with(&["cook-docs", "doctor", "--only", "bogus"], &[], |_| None)
            .unwrap_err();
        assert!(err.contains("bogus"), "err: {err}");
        assert!(err.contains("ignored"), "err: {err}");

        // Config file -> env -> CLI precedence for ignored_ingredients_file & ignored_ingredients
        let dir = std::env::temp_dir().join(format!("cook_docs_ign_cfg_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let cfg_file = dir.join(".cookdocs.toml");
        fs::write(
            &cfg_file,
            r#"
ignored_ingredients_file = "file_ignored.yaml"
ignored_ingredients = ["nonstick baking spray", "cooking spray"]
"#,
        )
        .unwrap();

        let from_file =
            load_config_with(&["cook-docs"], std::slice::from_ref(&cfg_file), |_| None).unwrap();
        assert_eq!(from_file.ignored_ingredients_file, "file_ignored.yaml");
        assert_eq!(
            from_file.ignored_ingredients,
            vec!["nonstick baking spray", "cooking spray"]
        );

        let from_env = load_config_with(
            &["cook-docs"],
            std::slice::from_ref(&cfg_file),
            |k| match k {
                "COOK_DOCS_IGNORED_INGREDIENTS_FILE" => Some("env_ignored.yaml".to_string()),
                "COOK_DOCS_IGNORED_INGREDIENTS" => Some("water, ice".to_string()),
                _ => None,
            },
        )
        .unwrap();
        assert_eq!(from_env.ignored_ingredients_file, "env_ignored.yaml");
        assert_eq!(from_env.ignored_ingredients, vec!["water", "ice"]);

        let from_cli = load_config_with(
            &[
                "cook-docs",
                "--ignored-ingredients-file",
                "cli_ignored.yaml",
            ],
            std::slice::from_ref(&cfg_file),
            |k| match k {
                "COOK_DOCS_IGNORED_INGREDIENTS_FILE" => Some("env_ignored.yaml".to_string()),
                _ => None,
            },
        )
        .unwrap();
        assert_eq!(from_cli.ignored_ingredients_file, "cli_ignored.yaml");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_link_in_steps_config_env_and_cli_precedence() {
        assert!(!Config::default().link_in_steps);

        let dir = std::env::temp_dir().join(format!("cook_docs_lis_cfg_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let toml_cfg = dir.join(".cookdocs.toml");
        fs::write(&toml_cfg, "link_in_steps = true\n").unwrap();
        let from_toml =
            load_config_with(&["cook-docs"], std::slice::from_ref(&toml_cfg), |_| None).unwrap();
        assert!(from_toml.link_in_steps);

        let yaml_cfg = dir.join(".cookdocs.yaml");
        fs::write(&yaml_cfg, "linkInSteps: true\n").unwrap();
        let from_yaml =
            load_config_with(&["cook-docs"], std::slice::from_ref(&yaml_cfg), |_| None).unwrap();
        assert!(from_yaml.link_in_steps);

        let json_cfg = dir.join(".cookdocs.json");
        fs::write(&json_cfg, r#"{"link-in-steps": true}"#).unwrap();
        let from_json =
            load_config_with(&["cook-docs"], std::slice::from_ref(&json_cfg), |_| None).unwrap();
        assert!(from_json.link_in_steps);

        // Env overrides config file
        let from_env = load_config_with(
            &["cook-docs"],
            std::slice::from_ref(&toml_cfg),
            |k| match k {
                "COOK_DOCS_LINK_IN_STEPS" => Some("false".to_string()),
                _ => None,
            },
        )
        .unwrap();
        assert!(!from_env.link_in_steps);

        // CLI overrides env
        let from_cli = load_config_with(
            &["cook-docs", "--link-in-steps"],
            std::slice::from_ref(&toml_cfg),
            |k| match k {
                "COOK_DOCS_LINK_IN_STEPS" => Some("false".to_string()),
                _ => None,
            },
        )
        .unwrap();
        assert!(from_cli.link_in_steps);

        let _ = fs::remove_dir_all(&dir);
    }
}

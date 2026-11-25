use crate::types::Config;
use figment::providers::{Format, Json, Toml, Yaml};
use ignore::gitignore::GitignoreBuilder;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static PROBE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckSeverity {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub category: String,
    pub severity: CheckSeverity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
    pub total_checks: usize,
    pub ok: usize,
    pub warnings: usize,
    pub errors: usize,
    pub healthy: bool,
}

impl DoctorReport {
    /// Build a `DoctorReport` summary from individual checks.
    pub fn new(checks: Vec<DoctorCheck>) -> Self {
        Self::with_strict(checks, false)
    }

    /// Build a `DoctorReport` summary, marking warnings as unhealthy when `strict` is enabled.
    pub fn with_strict(checks: Vec<DoctorCheck>, strict: bool) -> Self {
        let ok = checks
            .iter()
            .filter(|c| c.severity == CheckSeverity::Ok)
            .count();
        let warnings = checks
            .iter()
            .filter(|c| c.severity == CheckSeverity::Warning)
            .count();
        let errors = checks
            .iter()
            .filter(|c| c.severity == CheckSeverity::Error)
            .count();
        let total_checks = checks.len();
        let healthy = errors == 0 && (!strict || warnings == 0);
        Self {
            checks,
            total_checks,
            ok,
            warnings,
            errors,
            healthy,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecipeCheckKind {
    Syntax,
    Images,
    Metadata,
    Emojis,
    Ignored,
}

impl RecipeCheckKind {
    /// Parse a `--only` check category name (case-insensitive).
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "syntax" | "recipe_syntax" => Ok(Self::Syntax),
            "images" | "image" | "recipe_images" => Ok(Self::Images),
            "metadata" | "recipe_metadata" => Ok(Self::Metadata),
            "emojis" | "emoji" | "recipe_emojis" => Ok(Self::Emojis),
            "ignored" | "ignored_ingredients" | "recipe_ignored" => Ok(Self::Ignored),
            other => Err(format!(
                "Unknown --only check category '{other}': expected one of syntax, images, metadata, emojis, ignored"
            )),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DoctorOptions {
    pub check_recipes: bool,
    pub strict: bool,
    pub only: Vec<RecipeCheckKind>,
}

/// Run all diagnostic checks for `cook-docs`.
pub fn run_doctor(
    config_path: Option<&Path>,
    config_result: &Result<Config, String>,
) -> DoctorReport {
    run_doctor_with_options(config_path, config_result, DoctorOptions::default())
}

/// Run diagnostic checks for `cook-docs` with optional recipe-level validation (`--check-recipes`).
pub fn run_doctor_with_options(
    config_path: Option<&Path>,
    config_result: &Result<Config, String>,
    options: DoctorOptions,
) -> DoctorReport {
    let mut checks = Vec::new();

    match config_result {
        Ok(config) => {
            if let Some(path) = config_path {
                checks.push(DoctorCheck {
                    category: "config".into(),
                    severity: CheckSeverity::Ok,
                    message: format!("Configuration file valid: {}", path.display()),
                    path: Some(path.to_path_buf()),
                });
            } else {
                checks.push(DoctorCheck {
                    category: "config".into(),
                    severity: CheckSeverity::Ok,
                    message: "No custom configuration file found (using defaults)".into(),
                    path: None,
                });
            }

            let search_root = resolve_search_root(&config.recipe_search_root);
            let discovered =
                check_recipe_search_root(&search_root, &config.ignore_file, &mut checks);
            let recipes = if config.recipe_files.is_empty() {
                discovered
            } else {
                match crate::cook::resolve_recipe_paths(&search_root, config) {
                    Ok(targeted) => targeted,
                    Err(err) => {
                        checks.push(DoctorCheck {
                            category: "recipes".into(),
                            severity: CheckSeverity::Error,
                            message: err,
                            path: None,
                        });
                        Vec::new()
                    }
                }
            };
            checks.extend(check_ignore_file(&config.ignore_file));
            checks.extend(check_templates(
                &search_root,
                &recipes,
                &config.template_files,
            ));
            checks.extend(check_output_dir(&config.output_dir, &search_root));

            let emoji_table = check_emoji_config(config, &mut checks);
            check_weights_file(&config.weights_file, &mut checks);
            let ignored_ingredients = check_ignored_ingredients_config(config, &mut checks);

            if options.check_recipes || !options.only.is_empty() {
                checks.extend(check_recipes(
                    &recipes,
                    config,
                    emoji_table.as_ref().filter(|t| t.counts() != (0, 0)),
                    ignored_ingredients.as_ref().filter(|s| !s.is_empty()),
                    &options.only,
                ));
            }
        }
        Err(err) => {
            checks.push(DoctorCheck {
                category: "config".into(),
                severity: CheckSeverity::Error,
                message: err.clone(),
                path: config_path.map(Path::to_path_buf),
            });
        }
    }

    DoctorReport::with_strict(checks, options.strict)
}

fn check_emoji_config(
    config: &Config,
    checks: &mut Vec<DoctorCheck>,
) -> Option<crate::enrichment::EmojiTable> {
    let has_file = !config.emoji_file.trim().is_empty();
    let has_inline = config.emoji.is_some();
    if !has_file && !has_inline {
        return None;
    }

    let path = has_file.then(|| PathBuf::from(config.emoji_file.trim()));
    match crate::enrichment::EmojiTable::load(config) {
        Ok(table) => {
            let (ing_count, cw_count) = table.counts();
            checks.push(DoctorCheck {
                category: "emoji_file".into(),
                severity: CheckSeverity::Ok,
                message: format!(
                    "Emoji mappings valid ({ing_count} ingredient(s), {cw_count} cookware item(s))"
                ),
                path,
            });
            Some(table)
        }
        Err(err) => {
            checks.push(DoctorCheck {
                category: "emoji_file".into(),
                severity: CheckSeverity::Error,
                message: format!("Invalid emoji configuration: {err}"),
                path,
            });
            None
        }
    }
}

fn check_weights_file(weights_file: &str, checks: &mut Vec<DoctorCheck>) {
    if weights_file.trim().is_empty() {
        return;
    }
    let path = PathBuf::from(weights_file.trim());
    match crate::enrichment::WeightsTable::load(weights_file) {
        Ok(table) => checks.push(DoctorCheck {
            category: "weights_file".into(),
            severity: CheckSeverity::Ok,
            message: format!(
                "Weights file valid: {} ({} entry/entries)",
                path.display(),
                table.len()
            ),
            path: Some(path),
        }),
        Err(err) => checks.push(DoctorCheck {
            category: "weights_file".into(),
            severity: CheckSeverity::Error,
            message: format!("Invalid weights file: {err}"),
            path: Some(path),
        }),
    }
}

fn normalize_ingredient_key(raw: &str) -> String {
    raw.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn extract_string_list(value: &JsonValue, path: &Path) -> Result<Vec<String>, String> {
    let items = match value {
        JsonValue::Null => return Ok(Vec::new()),
        JsonValue::Array(arr) => arr,
        JsonValue::Object(map) => {
            if map.is_empty() {
                return Ok(Vec::new());
            }
            let found = [
                "ignored_ingredients",
                "ignored-ingredients",
                "ignoredIngredients",
                "ignored",
                "ingredients",
            ]
            .iter()
            .find_map(|k| map.get(*k));
            match found {
                Some(JsonValue::Array(arr)) => arr,
                Some(JsonValue::Null) => return Ok(Vec::new()),
                _ => {
                    return Err(format!(
                        "Expected a list of ingredient names (or 'ignored_ingredients' array) in '{}'",
                        path.display()
                    ));
                }
            }
        }
        _ => {
            return Err(format!(
                "Expected a list of ingredient names in '{}'",
                path.display()
            ));
        }
    };

    let mut out = Vec::with_capacity(items.len());
    for item in items {
        match item {
            JsonValue::String(s) => {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    out.push(trimmed.to_string());
                }
            }
            _ => {
                return Err(format!(
                    "Expected string entries in ignored ingredients file '{}'",
                    path.display()
                ));
            }
        }
    }
    Ok(out)
}

fn load_ignored_ingredients_file(path: &Path) -> Result<Vec<String>, String> {
    if !path.exists() {
        return Err(format!("File '{}' not found", path.display()));
    }
    if !path.is_file() {
        return Err(format!("Path '{}' is not a regular file", path.display()));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read '{}': {e}", path.display()))?;

    let value: JsonValue = match ext.as_deref() {
        Some("yaml") | Some("yml") => {
            let trimmed = content.trim();
            if trimmed.is_empty() || trimmed == "---" {
                JsonValue::Array(Vec::new())
            } else {
                Yaml::from_str(&content)
                    .map_err(|e| format!("Failed to parse '{}': {e}", path.display()))?
            }
        }
        Some("json") => Json::from_str(&content)
            .map_err(|e| format!("Failed to parse '{}': {e}", path.display()))?,
        Some("toml") => Toml::from_str(&content)
            .map_err(|e| format!("Failed to parse '{}': {e}", path.display()))?,
        _ => {
            return Err(format!(
                "Unsupported file format '{}': expected .toml, .yaml, .yml, or .json",
                path.display()
            ));
        }
    };

    extract_string_list(&value, path)
}

fn check_ignored_ingredients_config(
    config: &Config,
    checks: &mut Vec<DoctorCheck>,
) -> Option<HashSet<String>> {
    let has_file = !config.ignored_ingredients_file.trim().is_empty();
    let has_inline = config
        .ignored_ingredients
        .iter()
        .any(|s| !s.trim().is_empty());
    if !has_file && !has_inline {
        return None;
    }

    let path = has_file.then(|| PathBuf::from(config.ignored_ingredients_file.trim()));
    let mut normalized = HashSet::new();

    if let Some(ref file_path) = path {
        match load_ignored_ingredients_file(file_path) {
            Ok(items) => {
                for item in items {
                    let key = normalize_ingredient_key(&item);
                    if !key.is_empty() {
                        normalized.insert(key);
                    }
                }
            }
            Err(err) => {
                checks.push(DoctorCheck {
                    category: "ignored_ingredients_file".into(),
                    severity: CheckSeverity::Error,
                    message: format!("Invalid ignored ingredients configuration: {err}"),
                    path: Some(file_path.clone()),
                });
                return None;
            }
        }
    }

    for item in &config.ignored_ingredients {
        let key = normalize_ingredient_key(item);
        if !key.is_empty() {
            normalized.insert(key);
        }
    }

    checks.push(DoctorCheck {
        category: "ignored_ingredients_file".into(),
        severity: CheckSeverity::Ok,
        message: format!("Ignored ingredients valid ({} item(s))", normalized.len()),
        path,
    });
    Some(normalized)
}

fn check_recipes(
    recipes: &[PathBuf],
    config: &Config,
    emoji_table: Option<&crate::enrichment::EmojiTable>,
    ignored_ingredients: Option<&HashSet<String>>,
    only: &[RecipeCheckKind],
) -> Vec<DoctorCheck> {
    if recipes.is_empty() {
        return Vec::new();
    }

    let run_all = only.is_empty();
    let run_syntax = run_all || only.contains(&RecipeCheckKind::Syntax);
    let run_images = run_all || only.contains(&RecipeCheckKind::Images);
    let run_metadata = run_all || only.contains(&RecipeCheckKind::Metadata);
    let run_emojis = run_all || only.contains(&RecipeCheckKind::Emojis);
    let run_ignored = run_all || only.contains(&RecipeCheckKind::Ignored);
    let needs_parse = run_syntax || run_metadata || run_emojis || run_ignored;

    let mut syntax_issues = Vec::new();
    let mut image_issues = Vec::new();
    let mut metadata_issues = Vec::new();
    let mut emoji_issues = Vec::new();
    let mut ignored_issues = Vec::new();

    let mut syntax_errors = 0usize;
    let mut syntax_warnings = 0usize;
    let mut parsed_count = 0usize;

    let required_keys: Vec<&str> = config
        .required_metadata
        .iter()
        .map(String::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    for recipe_path in recipes {
        let recipe_str = recipe_path.to_string_lossy();
        let recipe_name = crate::cook::get_recipe_name(&recipe_str);

        if run_images && crate::cook::find_image_path(&recipe_str, &recipe_name).is_none() {
            image_issues.push(DoctorCheck {
                category: "recipe_images".into(),
                severity: CheckSeverity::Warning,
                message: format!(
                    "Missing companion image for '{}' ({})",
                    recipe_name,
                    recipe_path.display()
                ),
                path: Some(recipe_path.clone()),
            });
        }

        if !needs_parse {
            continue;
        }

        match crate::cook::validate_recipe_file(&recipe_str) {
            Ok(validation) => {
                parsed_count += 1;
                if run_syntax {
                    for err in &validation.errors {
                        syntax_errors += 1;
                        syntax_issues.push(DoctorCheck {
                            category: "recipe_syntax".into(),
                            severity: CheckSeverity::Error,
                            message: format!(
                                "Cooklang syntax error in '{}': {}",
                                recipe_path.display(),
                                err
                            ),
                            path: Some(recipe_path.clone()),
                        });
                    }
                    for warn_msg in &validation.warnings {
                        syntax_warnings += 1;
                        syntax_issues.push(DoctorCheck {
                            category: "recipe_syntax".into(),
                            severity: CheckSeverity::Warning,
                            message: format!(
                                "Cooklang warning in '{}': {}",
                                recipe_path.display(),
                                warn_msg
                            ),
                            path: Some(recipe_path.clone()),
                        });
                    }
                }

                if run_metadata && !required_keys.is_empty() {
                    let missing: Vec<&str> = required_keys
                        .iter()
                        .copied()
                        .filter(|req| {
                            !validation
                                .metadata
                                .iter()
                                .any(|(k, v)| k.eq_ignore_ascii_case(req) && !v.trim().is_empty())
                        })
                        .collect();
                    if !missing.is_empty() {
                        metadata_issues.push(DoctorCheck {
                            category: "recipe_metadata".into(),
                            severity: CheckSeverity::Error,
                            message: format!(
                                "Missing required metadata [{}] in '{}'",
                                missing.join(", "),
                                recipe_path.display()
                            ),
                            path: Some(recipe_path.clone()),
                        });
                    }
                }

                if run_emojis && let Some(table) = emoji_table {
                    let mut unmapped_ing = Vec::new();
                    for ing in &validation.ingredients {
                        if table.lookup_ingredient(ing).is_none()
                            && !unmapped_ing
                                .iter()
                                .any(|x: &String| x.eq_ignore_ascii_case(ing))
                        {
                            unmapped_ing.push(ing.clone());
                        }
                    }
                    let mut unmapped_cw = Vec::new();
                    for cw in &validation.cookware {
                        if table.lookup_cookware(cw).is_none()
                            && !unmapped_cw
                                .iter()
                                .any(|x: &String| x.eq_ignore_ascii_case(cw))
                        {
                            unmapped_cw.push(cw.clone());
                        }
                    }
                    if !unmapped_ing.is_empty() || !unmapped_cw.is_empty() {
                        let mut parts = Vec::new();
                        if !unmapped_ing.is_empty() {
                            parts.push(format!("ingredients: {}", unmapped_ing.join(", ")));
                        }
                        if !unmapped_cw.is_empty() {
                            parts.push(format!("cookware: {}", unmapped_cw.join(", ")));
                        }
                        emoji_issues.push(DoctorCheck {
                            category: "recipe_emojis".into(),
                            severity: CheckSeverity::Warning,
                            message: format!(
                                "Unmapped emoji(s) in '{}' ({})",
                                recipe_path.display(),
                                parts.join("; ")
                            ),
                            path: Some(recipe_path.clone()),
                        });
                    }
                }

                if run_ignored && let Some(ignored) = ignored_ingredients {
                    let mut flagged = Vec::new();
                    for ing in &validation.ingredients {
                        let key = normalize_ingredient_key(ing);
                        if ignored.contains(&key)
                            && !flagged
                                .iter()
                                .any(|x: &String| normalize_ingredient_key(x) == key)
                        {
                            flagged.push(ing.clone());
                        }
                    }
                    if !flagged.is_empty() {
                        ignored_issues.push(DoctorCheck {
                            category: "recipe_ignored".into(),
                            severity: CheckSeverity::Warning,
                            message: format!(
                                "Ignored ingredient(s) tagged with '@' in '{}' ({})",
                                recipe_path.display(),
                                flagged.join(", ")
                            ),
                            path: Some(recipe_path.clone()),
                        });
                    }
                }
            }
            Err(err) => {
                syntax_errors += 1;
                syntax_issues.push(DoctorCheck {
                    category: "recipe_syntax".into(),
                    severity: CheckSeverity::Error,
                    message: format!("Failed to validate '{}': {}", recipe_path.display(), err),
                    path: Some(recipe_path.clone()),
                });
            }
        }
    }

    let mut checks = Vec::new();
    if run_syntax {
        if syntax_errors == 0 && syntax_warnings == 0 {
            checks.push(DoctorCheck {
                category: "recipe_syntax".into(),
                severity: CheckSeverity::Ok,
                message: format!("Cooklang syntax valid for {} recipe(s)", recipes.len()),
                path: None,
            });
        } else {
            checks.extend(syntax_issues);
        }
    } else if !syntax_issues.is_empty() {
        // Surface unreadable recipe file I/O errors even when syntax warnings/errors are filtered out
        checks.extend(syntax_issues);
    }

    if run_images {
        if image_issues.is_empty() {
            checks.push(DoctorCheck {
                category: "recipe_images".into(),
                severity: CheckSeverity::Ok,
                message: format!(
                    "Companion images present for all {} recipe(s)",
                    recipes.len()
                ),
                path: None,
            });
        } else {
            checks.extend(image_issues);
        }
    }

    if run_metadata && !required_keys.is_empty() {
        if metadata_issues.is_empty() && parsed_count == recipes.len() {
            checks.push(DoctorCheck {
                category: "recipe_metadata".into(),
                severity: CheckSeverity::Ok,
                message: format!(
                    "Required metadata [{}] present in all {} recipe(s)",
                    required_keys.join(", "),
                    recipes.len()
                ),
                path: None,
            });
        } else {
            checks.extend(metadata_issues);
        }
    }

    if run_emojis && emoji_table.is_some() {
        if emoji_issues.is_empty() && parsed_count == recipes.len() {
            checks.push(DoctorCheck {
                category: "recipe_emojis".into(),
                severity: CheckSeverity::Ok,
                message: format!(
                    "All ingredients and cookware mapped to emojis across {} recipe(s)",
                    recipes.len()
                ),
                path: None,
            });
        } else {
            checks.extend(emoji_issues);
        }
    }

    if run_ignored && ignored_ingredients.is_some() {
        if ignored_issues.is_empty() && parsed_count == recipes.len() {
            checks.push(DoctorCheck {
                category: "recipe_ignored".into(),
                severity: CheckSeverity::Ok,
                message: format!(
                    "No ignored ingredients tagged with '@' across {} recipe(s)",
                    recipes.len()
                ),
                path: None,
            });
        } else {
            checks.extend(ignored_issues);
        }
    }

    checks
}

fn resolve_search_root(recipe_search_root: &str) -> PathBuf {
    let root = Path::new(recipe_search_root);
    if root.is_absolute() {
        root.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(root)
    }
}

fn check_recipe_search_root(
    search_root: &Path,
    ignore_file: &str,
    checks: &mut Vec<DoctorCheck>,
) -> Vec<PathBuf> {
    if !search_root.exists() {
        checks.push(DoctorCheck {
            category: "recipe_search_root".into(),
            severity: CheckSeverity::Error,
            message: format!(
                "Recipe search root does not exist: {}",
                search_root.display()
            ),
            path: Some(search_root.to_path_buf()),
        });
        return Vec::new();
    }

    if !search_root.is_dir() {
        checks.push(DoctorCheck {
            category: "recipe_search_root".into(),
            severity: CheckSeverity::Error,
            message: format!(
                "Recipe search root is not a directory: {}",
                search_root.display()
            ),
            path: Some(search_root.to_path_buf()),
        });
        return Vec::new();
    }

    let search_root_str = search_root.to_string_lossy();
    let recipes = crate::cook::find_recipe_file_paths(&search_root_str, ignore_file);
    if recipes.is_empty() {
        checks.push(DoctorCheck {
            category: "recipe_search_root".into(),
            severity: CheckSeverity::Warning,
            message: format!(
                "Recipe search root valid ({}), but 0 .cook files found",
                search_root.display()
            ),
            path: Some(search_root.to_path_buf()),
        });
    } else {
        checks.push(DoctorCheck {
            category: "recipe_search_root".into(),
            severity: CheckSeverity::Ok,
            message: format!(
                "Recipe search root valid ({}): {} recipe(s) found",
                search_root.display(),
                recipes.len()
            ),
            path: Some(search_root.to_path_buf()),
        });
    }
    recipes
}

fn check_ignore_file(ignore_file: &str) -> Vec<DoctorCheck> {
    if ignore_file.trim().is_empty() {
        return vec![DoctorCheck {
            category: "ignore_file".into(),
            severity: CheckSeverity::Ok,
            message: "Ignore file disabled (empty ignore_file setting)".into(),
            path: None,
        }];
    }

    let base_dir = crate::cook::find_git_repository_root().unwrap_or_else(|| {
        Path::new(".")
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from("."))
    });
    check_ignore_file_in(&base_dir, ignore_file)
}

fn check_ignore_file_in(base_dir: &Path, ignore_file: &str) -> Vec<DoctorCheck> {
    let ignore_path = base_dir.join(ignore_file);

    if !ignore_path.exists() {
        let (severity, msg) = if ignore_file == ".cookdocsignore" {
            (
                CheckSeverity::Ok,
                format!(
                    "No ignore file found at {} (optional)",
                    ignore_path.display()
                ),
            )
        } else {
            (
                CheckSeverity::Warning,
                format!(
                    "Configured ignore file not found: {}",
                    ignore_path.display()
                ),
            )
        };
        return vec![DoctorCheck {
            category: "ignore_file".into(),
            severity,
            message: msg,
            path: Some(ignore_path),
        }];
    }

    if !ignore_path.is_file() {
        return vec![DoctorCheck {
            category: "ignore_file".into(),
            severity: CheckSeverity::Error,
            message: format!(
                "Ignore file path is not a regular file: {}",
                ignore_path.display()
            ),
            path: Some(ignore_path),
        }];
    }

    let mut builder = GitignoreBuilder::new(base_dir);
    if let Some(err) = builder.add(&ignore_path) {
        return vec![DoctorCheck {
            category: "ignore_file".into(),
            severity: CheckSeverity::Error,
            message: format!(
                "Failed to parse ignore file '{}': {}",
                ignore_path.display(),
                err
            ),
            path: Some(ignore_path),
        }];
    }
    if let Err(err) = builder.build() {
        return vec![DoctorCheck {
            category: "ignore_file".into(),
            severity: CheckSeverity::Error,
            message: format!(
                "Invalid ignore rules in '{}': {}",
                ignore_path.display(),
                err
            ),
            path: Some(ignore_path),
        }];
    }

    vec![DoctorCheck {
        category: "ignore_file".into(),
        severity: CheckSeverity::Ok,
        message: format!("Ignore file valid: {}", ignore_path.display()),
        path: Some(ignore_path),
    }]
}

fn check_templates(
    search_root: &Path,
    recipes: &[PathBuf],
    template_files: &[String],
) -> Vec<DoctorCheck> {
    let mut checks = Vec::new();
    let search_root_str = search_root.to_string_lossy();

    // Check explicit non-base template paths for missing files
    for template_file in template_files {
        if !crate::cook::is_base_filename(template_file) {
            let resolved = crate::document::resolve_template_path(
                &search_root_str,
                &search_root.join("__probe__.cook").to_string_lossy(),
                template_file,
            );
            if !resolved.exists() || !resolved.is_file() {
                checks.push(DoctorCheck {
                    category: "templates".into(),
                    severity: CheckSeverity::Warning,
                    message: format!(
                        "Configured template file not found (will fall back to built-in default): {}",
                        resolved.display()
                    ),
                    path: Some(resolved),
                });
            }
        }
    }

    // Compile templates for all discovered recipes, or once against search_root when none exist
    let targets: Vec<PathBuf> = if recipes.is_empty() {
        vec![search_root.join("__probe__.cook")]
    } else {
        recipes.to_vec()
    };

    let mut compiled_count = 0usize;
    let mut had_error = false;
    for recipe_path in &targets {
        let recipe_str = recipe_path.to_string_lossy();
        match crate::document::build_documentation_template(
            &search_root_str,
            &recipe_str,
            template_files,
        ) {
            Ok(_) => compiled_count += 1,
            Err(err) => {
                had_error = true;
                checks.push(DoctorCheck {
                    category: "templates".into(),
                    severity: CheckSeverity::Error,
                    message: format!("Template compilation failed for {}: {}", recipe_str, err),
                    path: Some(recipe_path.clone()),
                });
            }
        }
    }

    if !had_error {
        let detail = if recipes.is_empty() {
            "built-in/configured templates valid".to_string()
        } else {
            format!("compiled templates for {} recipe(s)", compiled_count)
        };
        checks.push(DoctorCheck {
            category: "templates".into(),
            severity: CheckSeverity::Ok,
            message: format!("Templates valid ({detail})"),
            path: None,
        });
    }

    checks
}

fn check_dir_writable(dir: &Path) -> Result<(), String> {
    let id = PROBE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let probe = dir.join(format!(
        ".cook_docs_doctor_probe_{}_{}",
        std::process::id(),
        id
    ));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
        .map_err(|e| e.to_string())?;
    drop(file);
    let _ = fs::remove_file(&probe);
    Ok(())
}

fn check_output_dir(output_dir: &str, search_root: &Path) -> Vec<DoctorCheck> {
    if output_dir.trim().is_empty() {
        if !search_root.is_dir() {
            return Vec::new();
        }
        return match check_dir_writable(search_root) {
            Ok(()) => vec![DoctorCheck {
                category: "output_dir".into(),
                severity: CheckSeverity::Ok,
                message: format!(
                    "Output destination (in-place under {}) is writable",
                    search_root.display()
                ),
                path: Some(search_root.to_path_buf()),
            }],
            Err(err) => vec![DoctorCheck {
                category: "output_dir".into(),
                severity: CheckSeverity::Error,
                message: format!(
                    "Output destination (in-place under {}) is not writable: {}",
                    search_root.display(),
                    err
                ),
                path: Some(search_root.to_path_buf()),
            }],
        };
    }

    let out_path = Path::new(output_dir);
    if out_path.exists() {
        if !out_path.is_dir() {
            return vec![DoctorCheck {
                category: "output_dir".into(),
                severity: CheckSeverity::Error,
                message: format!("Output path is not a directory: {}", out_path.display()),
                path: Some(out_path.to_path_buf()),
            }];
        }
        return match check_dir_writable(out_path) {
            Ok(()) => vec![DoctorCheck {
                category: "output_dir".into(),
                severity: CheckSeverity::Ok,
                message: format!("Output directory is writable: {}", out_path.display()),
                path: Some(out_path.to_path_buf()),
            }],
            Err(err) => vec![DoctorCheck {
                category: "output_dir".into(),
                severity: CheckSeverity::Error,
                message: format!(
                    "Output directory is not writable ({}): {}",
                    out_path.display(),
                    err
                ),
                path: Some(out_path.to_path_buf()),
            }],
        };
    }

    // Walk ancestors to find nearest existing directory for writability check
    let mut ancestor = out_path.parent();
    while let Some(p) = ancestor {
        if p.as_os_str().is_empty() {
            let dot = Path::new(".");
            return match check_dir_writable(dot) {
                Ok(()) => vec![DoctorCheck {
                    category: "output_dir".into(),
                    severity: CheckSeverity::Ok,
                    message: format!(
                        "Output directory '{}' does not exist yet (working directory is writable)",
                        out_path.display()
                    ),
                    path: Some(out_path.to_path_buf()),
                }],
                Err(err) => vec![DoctorCheck {
                    category: "output_dir".into(),
                    severity: CheckSeverity::Error,
                    message: format!(
                        "Output directory '{}' cannot be created (working directory not writable: {})",
                        out_path.display(),
                        err
                    ),
                    path: Some(out_path.to_path_buf()),
                }],
            };
        }
        if p.exists() {
            if !p.is_dir() {
                return vec![DoctorCheck {
                    category: "output_dir".into(),
                    severity: CheckSeverity::Error,
                    message: format!(
                        "Output directory ancestor '{}' is not a directory",
                        p.display()
                    ),
                    path: Some(p.to_path_buf()),
                }];
            }
            return match check_dir_writable(p) {
                Ok(()) => vec![DoctorCheck {
                    category: "output_dir".into(),
                    severity: CheckSeverity::Ok,
                    message: format!(
                        "Output directory '{}' does not exist yet (parent '{}' is writable)",
                        out_path.display(),
                        p.display()
                    ),
                    path: Some(out_path.to_path_buf()),
                }],
                Err(err) => vec![DoctorCheck {
                    category: "output_dir".into(),
                    severity: CheckSeverity::Error,
                    message: format!(
                        "Output directory '{}' cannot be created (parent '{}' not writable: {})",
                        out_path.display(),
                        p.display(),
                        err
                    ),
                    path: Some(out_path.to_path_buf()),
                }],
            };
        }
        ancestor = p.parent();
    }

    vec![DoctorCheck {
        category: "output_dir".into(),
        severity: CheckSeverity::Error,
        message: format!(
            "Cannot resolve writable parent for output directory '{}'",
            out_path.display()
        ),
        path: Some(out_path.to_path_buf()),
    }]
}

/// Format a `DoctorReport` into a human-readable report string.
pub fn format_doctor_report(report: &DoctorReport, colorize: bool) -> String {
    let mut out = String::new();

    for check in &report.checks {
        let tag = match check.severity {
            CheckSeverity::Ok => {
                if colorize {
                    "\x1b[32m[OK]\x1b[0m   "
                } else {
                    "[OK]   "
                }
            }
            CheckSeverity::Warning => {
                if colorize {
                    "\x1b[33m[WARN]\x1b[0m "
                } else {
                    "[WARN] "
                }
            }
            CheckSeverity::Error => {
                if colorize {
                    "\x1b[31m[ERROR]\x1b[0m"
                } else {
                    "[ERROR]"
                }
            }
        };
        out.push_str(&format!("{} {}\n", tag, check.message));
    }

    out.push_str(&format!(
        "\nDoctor summary: {} passed, {} warning(s), {} error(s).\n",
        report.ok, report.warnings, report.errors
    ));

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir(prefix: &str) -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "cook_docs_doc_{}_{}_{}",
            prefix,
            std::process::id(),
            id
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_doctor_report_counts_and_healthy() {
        let checks = vec![
            DoctorCheck {
                category: "config".into(),
                severity: CheckSeverity::Ok,
                message: "ok".into(),
                path: None,
            },
            DoctorCheck {
                category: "templates".into(),
                severity: CheckSeverity::Warning,
                message: "warn".into(),
                path: None,
            },
        ];
        let report = DoctorReport::new(checks);
        assert_eq!(report.total_checks, 2);
        assert_eq!(report.ok, 1);
        assert_eq!(report.warnings, 1);
        assert_eq!(report.errors, 0);
        assert!(report.healthy);

        let report_err = DoctorReport::new(vec![DoctorCheck {
            category: "config".into(),
            severity: CheckSeverity::Error,
            message: "err".into(),
            path: None,
        }]);
        assert_eq!(report_err.errors, 1);
        assert!(!report_err.healthy);
    }

    #[test]
    fn test_format_doctor_report_color_and_plain() {
        let report = DoctorReport::new(vec![
            DoctorCheck {
                category: "config".into(),
                severity: CheckSeverity::Ok,
                message: "Config ok".into(),
                path: None,
            },
            DoctorCheck {
                category: "ignore_file".into(),
                severity: CheckSeverity::Warning,
                message: "Ignore warn".into(),
                path: None,
            },
            DoctorCheck {
                category: "templates".into(),
                severity: CheckSeverity::Error,
                message: "Template err".into(),
                path: None,
            },
        ]);

        let plain = format_doctor_report(&report, false);
        assert!(plain.contains("[OK]    Config ok"));
        assert!(plain.contains("[WARN]  Ignore warn"));
        assert!(plain.contains("[ERROR] Template err"));
        assert!(plain.contains("Doctor summary: 1 passed, 1 warning(s), 1 error(s)."));

        let colored = format_doctor_report(&report, true);
        assert!(colored.contains("\x1b[32m[OK]\x1b[0m"));
        assert!(colored.contains("\x1b[33m[WARN]\x1b[0m"));
        assert!(colored.contains("\x1b[31m[ERROR]\x1b[0m"));
    }

    #[test]
    fn test_run_doctor_healthy_and_error_states() {
        let dir = temp_dir("run");
        let recipes = dir.join("recipes");
        let out = dir.join("out/nested");
        fs::create_dir_all(&recipes).unwrap();
        fs::write(recipes.join("Dish.cook"), "Boil @water{1%l}.").unwrap();

        let cfg = Config {
            ignore_file: "".to_string(),
            recipe_search_root: recipes.to_string_lossy().to_string(),
            output_dir: out.to_string_lossy().to_string(),
            ..Config::default()
        };

        let report_ok = run_doctor(None, &Ok(cfg.clone()));
        assert!(report_ok.healthy);
        assert_eq!(report_ok.errors, 0);

        let bad_cfg = Config {
            recipe_search_root: dir.join("missing_root").to_string_lossy().to_string(),
            ..cfg
        };
        let report_bad = run_doctor(None, &Ok(bad_cfg));
        assert!(!report_bad.healthy);
        assert!(report_bad.errors >= 1);

        let report_cfg_err = run_doctor(
            Some(&dir.join("bad.toml")),
            &Err("Failed to parse config".to_string()),
        );
        assert!(!report_cfg_err.healthy);
        assert_eq!(report_cfg_err.checks.len(), 1);
        assert_eq!(report_cfg_err.checks[0].category, "config");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_search_root_and_recipe_root_edge_cases() {
        let rel = resolve_search_root("testdata");
        assert!(rel.is_absolute());
        assert!(rel.ends_with("testdata"));

        let dir = temp_dir("root_edges");
        let file_root = dir.join("not_a_dir.cook");
        fs::write(&file_root, "").unwrap();

        let mut checks = Vec::new();
        let found = check_recipe_search_root(&file_root, "", &mut checks);
        assert!(found.is_empty());
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].severity, CheckSeverity::Error);
        assert!(checks[0].message.contains("not a directory"));

        let empty_dir = dir.join("empty");
        fs::create_dir_all(&empty_dir).unwrap();
        let mut empty_checks = Vec::new();
        let empty_found = check_recipe_search_root(&empty_dir, "", &mut empty_checks);
        assert!(empty_found.is_empty());
        assert_eq!(empty_checks.len(), 1);
        assert_eq!(empty_checks[0].severity, CheckSeverity::Warning);
        assert!(empty_checks[0].message.contains("0 .cook files"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_check_ignore_file_all_variants() {
        let dir = temp_dir("ignore_variants");

        // 1. Default missing .cookdocsignore is optional (Ok)
        let missing_default = check_ignore_file_in(&dir, ".cookdocsignore");
        assert_eq!(missing_default[0].severity, CheckSeverity::Ok);
        assert!(missing_default[0].message.contains("optional"));

        // 2. Missing custom ignore file is a Warning
        let missing_custom = check_ignore_file_in(&dir, ".myignore");
        assert_eq!(missing_custom[0].severity, CheckSeverity::Warning);
        assert!(missing_custom[0].message.contains("not found"));

        // 3. Directory instead of regular file is an Error
        let as_dir = dir.join("dir_ignore");
        fs::create_dir_all(&as_dir).unwrap();
        let dir_res = check_ignore_file_in(&dir, "dir_ignore");
        assert_eq!(dir_res[0].severity, CheckSeverity::Error);
        assert!(dir_res[0].message.contains("not a regular file"));

        // 4. Non-UTF8 ignore file fails parsing with Error
        let bad_file = dir.join(".badignore");
        fs::write(&bad_file, b"\xff\xfe\n").unwrap();
        let bad_res = check_ignore_file_in(&dir, ".badignore");
        assert_eq!(bad_res[0].severity, CheckSeverity::Error);
        assert!(bad_res[0].message.contains("Failed to parse ignore file"));

        // 5. Valid ignore file is Ok
        let good_file = dir.join(".goodignore");
        fs::write(&good_file, "drafts/\n").unwrap();
        let good_res = check_ignore_file_in(&dir, ".goodignore");
        assert_eq!(good_res[0].severity, CheckSeverity::Ok);
        assert!(good_res[0].message.contains("Ignore file valid"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_check_templates_missing_explicit_and_empty_recipes() {
        let dir = temp_dir("tpl_edges");

        let checks = check_templates(
            &dir,
            &[],
            &[
                "./missing.gotmpl".to_string(),
                "recipe.md.gotmpl".to_string(),
            ],
        );
        assert!(checks.iter().any(|c| {
            c.severity == CheckSeverity::Warning
                && c.message.contains("Configured template file not found")
        }));
        assert!(checks.iter().any(|c| {
            c.severity == CheckSeverity::Ok
                && c.message.contains("built-in/configured templates valid")
        }));

        // Broken template compilation error
        fs::write(dir.join("recipe.md.gotmpl"), "{{ .Unclosed").unwrap();
        let err_checks = check_templates(&dir, &[], &["recipe.md.gotmpl".to_string()]);
        assert!(
            err_checks
                .iter()
                .any(|c| c.severity == CheckSeverity::Error)
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_check_output_dir_all_variants() {
        let dir = temp_dir("out_variants");
        let file_path = dir.join("regular_file");
        fs::write(&file_path, "x").unwrap();

        // 1. Empty output_dir with non-directory search_root returns no checks
        assert!(check_output_dir("", &file_path).is_empty());

        // 2. Empty output_dir with writable search_root returns Ok in-place
        let in_place = check_output_dir("", &dir);
        assert_eq!(in_place[0].severity, CheckSeverity::Ok);
        assert!(in_place[0].message.contains("in-place"));

        // 3. Existing file as output_dir returns Error
        let file_out = check_output_dir(&file_path.to_string_lossy(), &dir);
        assert_eq!(file_out[0].severity, CheckSeverity::Error);
        assert!(file_out[0].message.contains("not a directory"));

        // 4. Existing writable directory returns Ok
        let existing_out = check_output_dir(&dir.to_string_lossy(), &dir);
        assert_eq!(existing_out[0].severity, CheckSeverity::Ok);
        assert!(existing_out[0].message.contains("is writable"));

        // 5. Bare non-existent relative name checks working directory "."
        let rel_out = check_output_dir("nonexistent_doctor_rel_out_xyz", &dir);
        assert_eq!(rel_out[0].severity, CheckSeverity::Ok);
        assert!(rel_out[0].message.contains("working directory is writable"));

        // 6. Non-existent output_dir whose ancestor is a regular file returns Error
        let under_file = file_path.join("nested/out");
        let bad_ancestor = check_output_dir(&under_file.to_string_lossy(), &dir);
        assert_eq!(bad_ancestor[0].severity, CheckSeverity::Error);
        assert!(bad_ancestor[0].message.contains("ancestor"));

        // 7. Read-only directory checks (Unix non-root)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let ro_dir = dir.join("readonly");
            fs::create_dir_all(&ro_dir).unwrap();
            fs::set_permissions(&ro_dir, fs::Permissions::from_mode(0o555)).unwrap();

            if check_dir_writable(&ro_dir).is_err() {
                let ro_in_place = check_output_dir("", &ro_dir);
                assert_eq!(ro_in_place[0].severity, CheckSeverity::Error);
                assert!(ro_in_place[0].message.contains("not writable"));

                let ro_existing = check_output_dir(&ro_dir.to_string_lossy(), &dir);
                assert_eq!(ro_existing[0].severity, CheckSeverity::Error);
                assert!(ro_existing[0].message.contains("not writable"));

                let ro_nested = ro_dir.join("sub/out");
                let ro_parent = check_output_dir(&ro_nested.to_string_lossy(), &dir);
                assert_eq!(ro_parent[0].severity, CheckSeverity::Error);
                assert!(ro_parent[0].message.contains("not writable"));
            }

            let _ = fs::set_permissions(&ro_dir, fs::Permissions::from_mode(0o755));
        }

        // Also cover run_doctor with explicit Some(config_path)
        let cfg = Config {
            ignore_file: "".to_string(),
            recipe_search_root: dir.to_string_lossy().to_string(),
            ..Config::default()
        };
        let cfg_file = dir.join(".cookdocs.toml");
        let rep = run_doctor(Some(&cfg_file), &Ok(cfg));
        assert!(rep.healthy);
        assert!(rep.checks[0].message.contains("Configuration file valid:"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_run_doctor_check_recipes_healthy_and_failing() {
        let dir = temp_dir("check_recipes");
        let recipes_dir = dir.join("cook");
        fs::create_dir_all(&recipes_dir).unwrap();

        let emoji_path = dir.join("emoji.yaml");
        fs::write(
            &emoji_path,
            "emoji:\n  ingredients:\n    - herb:\n        - cilantro\n  cookware:\n    - bowl_with_spoon:\n        - bowl\n",
        )
        .unwrap();
        let weights_path = dir.join("weights.yaml");
        fs::write(&weights_path, "cilantro: 16\n").unwrap();

        let good_recipe = recipes_dir.join("Salsa.cook");
        fs::write(
            &good_recipe,
            "---\nsource: https://example.com/salsa\nserves: 4\n---\nChop @cilantro{1%cup} in #bowl{}.\n",
        )
        .unwrap();
        fs::write(recipes_dir.join("Salsa.webp"), b"webp").unwrap();

        let healthy_cfg = Config {
            ignore_file: String::new(),
            recipe_search_root: recipes_dir.to_string_lossy().to_string(),
            emoji_file: emoji_path.to_string_lossy().to_string(),
            weights_file: weights_path.to_string_lossy().to_string(),
            required_metadata: vec!["source".to_string(), "serves".to_string()],
            ..Config::default()
        };
        let report_ok = run_doctor_with_options(
            None,
            &Ok(healthy_cfg.clone()),
            DoctorOptions {
                check_recipes: true,
                ..DoctorOptions::default()
            },
        );
        assert!(report_ok.healthy, "report: {report_ok:#?}");
        assert_eq!(report_ok.warnings, 0);
        assert!(
            report_ok
                .checks
                .iter()
                .any(|c| c.category == "recipe_syntax" && c.severity == CheckSeverity::Ok)
        );
        assert!(
            report_ok
                .checks
                .iter()
                .any(|c| c.category == "recipe_images" && c.severity == CheckSeverity::Ok)
        );
        assert!(
            report_ok
                .checks
                .iter()
                .any(|c| c.category == "recipe_metadata" && c.severity == CheckSeverity::Ok)
        );
        assert!(
            report_ok
                .checks
                .iter()
                .any(|c| c.category == "recipe_emojis" && c.severity == CheckSeverity::Ok)
        );

        // Add recipe with syntax error (invalid timer unit), warning (bad YAML frontmatter), and missing image
        let bad_recipe = recipes_dir.join("Broken.cook");
        fs::write(
            &bad_recipe,
            "---\ntitle: [unclosed\n---\nBake @dragonfruit{1%cup} in #cauldron{} for ~{10%bogusunit}.\n",
        )
        .unwrap();

        let targeted_cfg = Config {
            recipe_files: vec![bad_recipe.to_string_lossy().to_string()],
            ..healthy_cfg
        };
        let report_bad = run_doctor_with_options(
            None,
            &Ok(targeted_cfg),
            DoctorOptions {
                check_recipes: true,
                ..DoctorOptions::default()
            },
        );
        assert!(!report_bad.healthy);
        assert!(
            report_bad
                .checks
                .iter()
                .any(|c| c.category == "recipe_syntax" && c.severity == CheckSeverity::Error)
        );
        assert!(
            report_bad
                .checks
                .iter()
                .any(|c| c.category == "recipe_syntax" && c.severity == CheckSeverity::Warning)
        );
        assert!(
            report_bad
                .checks
                .iter()
                .any(|c| c.category == "recipe_images" && c.severity == CheckSeverity::Warning)
        );

        // Recipe with valid syntax, missing image, missing metadata, unmapped ingredient/cookware
        let warn_recipe = recipes_dir.join("Incomplete.cook");
        fs::write(&warn_recipe, "Mix @dragonfruit{1%cup} in #cauldron{}.\n").unwrap();
        let warn_cfg = Config {
            ignore_file: String::new(),
            recipe_search_root: recipes_dir.to_string_lossy().to_string(),
            emoji_file: emoji_path.to_string_lossy().to_string(),
            required_metadata: vec!["source".to_string()],
            recipe_files: vec![warn_recipe.to_string_lossy().to_string()],
            ..Config::default()
        };
        let report_warn = run_doctor_with_options(
            None,
            &Ok(warn_cfg),
            DoctorOptions {
                check_recipes: true,
                ..DoctorOptions::default()
            },
        );
        assert!(!report_warn.healthy);
        assert!(
            report_warn
                .checks
                .iter()
                .any(|c| c.category == "recipe_metadata" && c.severity == CheckSeverity::Error)
        );
        assert!(
            report_warn
                .checks
                .iter()
                .any(|c| c.category == "recipe_emojis"
                    && c.severity == CheckSeverity::Warning
                    && c.message.contains("dragonfruit")
                    && c.message.contains("cauldron"))
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_doctor_only_filter_strict_and_ignored_ingredients() {
        let dir = temp_dir("only_strict_ignored");
        let recipes_dir = dir.join("cook");
        fs::create_dir_all(&recipes_dir).unwrap();

        // Cooklang-compatible top-level YAML list
        let ignored_yaml = dir.join("ignored_ingredients.yaml");
        fs::write(&ignored_yaml, "---\n- nonstick baking spray\n").unwrap();

        // TOML table format
        let ignored_toml = dir.join("ignored_ingredients.toml");
        fs::write(&ignored_toml, "ignored_ingredients = [\"cooking spray\"]\n").unwrap();

        // JSON top-level array format
        let ignored_json = dir.join("ignored_ingredients.json");
        fs::write(&ignored_json, r#"["parchment paper"]"#).unwrap();

        let emoji_path = dir.join("emoji.yaml");
        fs::write(
            &emoji_path,
            "emoji:\n  ingredients:\n    - bread:\n        - flour\n",
        )
        .unwrap();

        // Recipe with legacy `>>` syntax warning, missing image, missing metadata, unmapped emoji,
        // and an ignored ingredient tagged with `@` (mixed casing + extra spaces)
        let noisy_recipe = recipes_dir.join("Brownies.cook");
        fs::write(
            &noisy_recipe,
            ">> title: Brownies\nCoat pan with @Nonstick  Baking Spray{} and add @cocoa{1%cup}.\n",
        )
        .unwrap();

        let base_cfg = Config {
            ignore_file: String::new(),
            recipe_search_root: recipes_dir.to_string_lossy().to_string(),
            emoji_file: emoji_path.to_string_lossy().to_string(),
            required_metadata: vec!["source".to_string()],
            ignored_ingredients_file: ignored_yaml.to_string_lossy().to_string(),
            ..Config::default()
        };

        // 1. --only images emits only recipe_images (no syntax/metadata/emojis/ignored)
        let rep_images = run_doctor_with_options(
            None,
            &Ok(base_cfg.clone()),
            DoctorOptions {
                check_recipes: true,
                strict: false,
                only: vec![RecipeCheckKind::Images],
            },
        );
        assert!(
            rep_images.healthy,
            "warnings alone stay healthy without --strict"
        );
        assert!(rep_images.warnings > 0);
        assert!(
            rep_images
                .checks
                .iter()
                .any(|c| c.category == "recipe_images" && c.severity == CheckSeverity::Warning)
        );
        assert!(!rep_images.checks.iter().any(|c| matches!(
            c.category.as_str(),
            "recipe_syntax" | "recipe_metadata" | "recipe_emojis" | "recipe_ignored"
        )));

        // 2. --strict turns warnings into unhealthy
        let rep_images_strict = run_doctor_with_options(
            None,
            &Ok(base_cfg.clone()),
            DoctorOptions {
                check_recipes: true,
                strict: true,
                only: vec![RecipeCheckKind::Images],
            },
        );
        assert!(!rep_images_strict.healthy);
        assert_eq!(rep_images_strict.errors, 0);
        assert!(rep_images_strict.warnings > 0);

        // 3. --only ignored detects `@Nonstick  Baking Spray{}` via YAML list
        let rep_ignored = run_doctor_with_options(
            None,
            &Ok(base_cfg.clone()),
            DoctorOptions {
                check_recipes: true,
                strict: true,
                only: vec![RecipeCheckKind::Ignored],
            },
        );
        assert!(!rep_ignored.healthy);
        assert!(
            rep_ignored.checks.iter().any(|c| {
                c.category == "recipe_ignored"
                    && c.severity == CheckSeverity::Warning
                    && c.message.contains("Nonstick Baking Spray")
            }),
            "checks: {:#?}",
            rep_ignored.checks
        );
        assert!(!rep_ignored.checks.iter().any(|c| matches!(
            c.category.as_str(),
            "recipe_syntax" | "recipe_images" | "recipe_metadata" | "recipe_emojis"
        )));

        // 4. TOML + JSON + empty YAML + inline ignored_ingredients all work and clean recipe passes --strict
        let empty_yaml = dir.join("empty_ignored.yaml");
        fs::write(&empty_yaml, "---\n").unwrap();
        let clean_recipe = recipes_dir.join("Clean.cook");
        fs::write(
            &clean_recipe,
            "---\nsource: home\n---\nMix @flour{1%cup}.\n",
        )
        .unwrap();
        for file_path in [&ignored_toml, &ignored_json, &empty_yaml] {
            let cfg = Config {
                ignore_file: String::new(),
                recipe_search_root: recipes_dir.to_string_lossy().to_string(),
                recipe_files: vec![clean_recipe.to_string_lossy().to_string()],
                ignored_ingredients_file: file_path.to_string_lossy().to_string(),
                ignored_ingredients: vec!["ice".to_string()],
                ..Config::default()
            };
            let rep = run_doctor_with_options(
                None,
                &Ok(cfg),
                DoctorOptions {
                    check_recipes: true,
                    strict: true,
                    only: vec![RecipeCheckKind::Ignored],
                },
            );
            assert!(rep.healthy, "file {:?} report: {rep:#?}", file_path);
            assert!(
                rep.checks
                    .iter()
                    .any(|c| c.category == "recipe_ignored" && c.severity == CheckSeverity::Ok)
            );
        }

        // 5. Missing, unsupported-ext, and invalid-schema ignored_ingredients_file report errors
        let bad_ext = dir.join("ignored.txt");
        fs::write(&bad_ext, "water\n").unwrap();
        let bad_schema = dir.join("bad_ignored.json");
        fs::write(&bad_schema, "[123]").unwrap();
        let bad_obj = dir.join("bad_obj.toml");
        fs::write(&bad_obj, "other = [\"water\"]\n").unwrap();
        for bad_path in [dir.join("missing.yaml"), bad_ext, bad_schema, bad_obj] {
            let bad_cfg = Config {
                ignore_file: String::new(),
                recipe_search_root: recipes_dir.to_string_lossy().to_string(),
                ignored_ingredients_file: bad_path.to_string_lossy().to_string(),
                ..Config::default()
            };
            let rep_bad = run_doctor_with_options(None, &Ok(bad_cfg), DoctorOptions::default());
            assert!(!rep_bad.healthy, "expected error for {:?}", bad_path);
            assert!(rep_bad.checks.iter().any(|c| {
                c.category == "ignored_ingredients_file" && c.severity == CheckSeverity::Error
            }));
        }

        // 6. RecipeCheckKind::parse aliases & error
        assert_eq!(
            RecipeCheckKind::parse("syntax").unwrap(),
            RecipeCheckKind::Syntax
        );
        assert_eq!(
            RecipeCheckKind::parse("image").unwrap(),
            RecipeCheckKind::Images
        );
        assert_eq!(
            RecipeCheckKind::parse("recipe_metadata").unwrap(),
            RecipeCheckKind::Metadata
        );
        assert_eq!(
            RecipeCheckKind::parse("emoji").unwrap(),
            RecipeCheckKind::Emojis
        );
        assert_eq!(
            RecipeCheckKind::parse("ignored_ingredients").unwrap(),
            RecipeCheckKind::Ignored
        );
        assert!(RecipeCheckKind::parse("nope").is_err());

        let _ = fs::remove_dir_all(&dir);
    }
}

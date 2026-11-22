use ignore::gitignore::GitignoreBuilder;
use log::{debug, warn};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub fn find_git_repository_root() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if output.status.success() {
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(PathBuf::from(path_str))
    } else {
        None
    }
}

pub fn is_relative_path(file_path: &str) -> bool {
    file_path.starts_with('.')
        && Path::new(file_path).file_name().and_then(|f| f.to_str()) != Some(file_path)
}

pub fn is_base_filename(file_path: &str) -> bool {
    Path::new(file_path).file_name().and_then(|f| f.to_str()) == Some(file_path)
}

pub fn get_recipe_name(recipe_path: &str) -> String {
    Path::new(recipe_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

pub fn slugify_filename(name: &str) -> String {
    name.replace(' ', "-").to_lowercase()
}

pub fn build_image_slug(image_file_name: &str, image_ext_override: &str) -> String {
    if image_file_name.is_empty() {
        return String::new();
    }
    let slug = slugify_filename(image_file_name);
    let ext = image_ext_override
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    if ext.is_empty() {
        return slug;
    }
    let stem = Path::new(&slug)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&slug);
    format!("{stem}.{ext}")
}

fn relative_recipe_dir(recipe_path: &str, config: &crate::types::Config) -> Option<PathBuf> {
    let search_root = if config.recipe_search_root.is_empty() {
        "."
    } else {
        &config.recipe_search_root
    };

    let abs_search_root = Path::new(search_root)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(search_root));
    let abs_recipe_path = Path::new(recipe_path)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(recipe_path));

    abs_recipe_path
        .strip_prefix(&abs_search_root)
        .ok()
        .map(|rel| rel.parent().unwrap_or_else(|| Path::new("")).to_path_buf())
}

pub fn get_new_recipe_path(
    recipe_path: &str,
    recipe_name: &str,
    config: &crate::types::Config,
) -> PathBuf {
    let file_name = format!("{}.md", slugify_filename(recipe_name));

    if config.output_dir.is_empty() {
        let parent = Path::new(recipe_path)
            .parent()
            .unwrap_or_else(|| Path::new(""));
        return parent.join(file_name);
    }

    match relative_recipe_dir(recipe_path, config) {
        Some(rel_dir) => Path::new(&config.output_dir).join(rel_dir).join(file_name),
        None => Path::new(&config.output_dir).join(file_name),
    }
}

/// Quietly search for a companion image alongside `recipe_path`.
pub fn find_image_path(recipe_path: &str, recipe_name: &str) -> Option<PathBuf> {
    let parent = Path::new(recipe_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));

    for ext in ["webp", "jpg", "jpeg", "png"] {
        let candidate = parent.join(format!("{}.{}", recipe_name, ext));
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

pub fn get_image_path(recipe_path: &str, recipe_name: &str) -> Option<PathBuf> {
    if let Some(found) = find_image_path(recipe_path, recipe_name) {
        return Some(found);
    }

    let parent = Path::new(recipe_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    warn!("Image file {} missing.", parent.join(recipe_name).display());
    None
}

pub fn find_recipe_file_paths(recipe_search_root: &str, ignore_file: &str) -> Vec<PathBuf> {
    let mut recipe_paths = Vec::new();

    // Build gitignore matcher
    let relative_dir = match find_git_repository_root() {
        Some(root) => root.canonicalize().unwrap_or(root),
        None => Path::new(".")
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(".")),
    };

    let mut gitignore_builder = GitignoreBuilder::new(&relative_dir);
    if !ignore_file.is_empty() {
        let ignore_path = relative_dir.join(ignore_file);
        if ignore_path.exists() {
            if let Some(err) = gitignore_builder.add(&ignore_path) {
                warn!("Error loading ignore file {:?}: {:?}", ignore_path, err);
            } else {
                debug!("Loaded ignore rules from {:?}", ignore_path);
            }
        }
    }
    let gitignore = gitignore_builder
        .build()
        .unwrap_or_else(|_| ignore::gitignore::Gitignore::empty());

    let walk = WalkDir::new(recipe_search_root).into_iter();
    for entry in walk
        .filter_entry(|e| {
            let path = e.path();

            // Exclude git folder
            if path.ends_with(".git") {
                return false;
            }

            // Get relative path for ignore checking (canonicalize so short/relative prefixes match)
            let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            if canonical_path
                .strip_prefix(&relative_dir)
                .map(|rel| gitignore.matched(rel, path.is_dir()).is_ignore())
                .unwrap_or(false)
            {
                debug!("Ignoring directory or file {:?}", path);
                return false;
            }
            true
        })
        .flatten()
    {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "cook") {
            recipe_paths.push(path.to_path_buf());
        }
    }

    recipe_paths.sort();
    recipe_paths
}

pub fn resolve_recipe_paths(
    search_root: &Path,
    config: &crate::types::Config,
) -> Result<Vec<PathBuf>, String> {
    if config.recipe_files.is_empty() {
        return Ok(find_recipe_file_paths(
            &search_root.to_string_lossy(),
            &config.ignore_file,
        ));
    }

    let mut seen = HashSet::new();
    let mut recipe_paths = Vec::new();
    let mut push_unique = |path: PathBuf| {
        let key = path.canonicalize().unwrap_or_else(|_| path.clone());
        if seen.insert(key) {
            recipe_paths.push(path);
        }
    };

    for raw in &config.recipe_files {
        let direct = PathBuf::from(raw);
        let resolved = if direct.exists() {
            direct
        } else {
            let joined = search_root.join(raw);
            if joined.exists() {
                joined
            } else {
                return Err(format!("Recipe path '{}' not found", raw));
            }
        };

        if resolved.is_file() {
            if resolved.extension().is_some_and(|ext| ext == "cook") {
                push_unique(resolved);
            } else {
                return Err(format!("Recipe file '{}' must have a .cook extension", raw));
            }
        } else if resolved.is_dir() {
            for found in find_recipe_file_paths(&resolved.to_string_lossy(), &config.ignore_file) {
                push_unique(found);
            }
        } else {
            return Err(format!("Recipe path '{}' is not a file or directory", raw));
        }
    }

    Ok(recipe_paths)
}

/// Structured diagnostics and extracted recipe facts from validating a `.cook` file.
#[derive(Debug, Clone, Default)]
pub struct RecipeValidation {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
    pub ingredients: Vec<String>,
    pub cookware: Vec<String>,
}

fn format_cooklang_diag(content: &str, diag: &cooklang::error::SourceDiag) -> String {
    if let Some((span, _)) = diag.labels.first() {
        let offset = span.start().min(content.len());
        let line = content.as_bytes()[..offset]
            .iter()
            .filter(|&&b| b == b'\n')
            .count()
            + 1;
        format!("line {line}: {}", diag.message)
    } else {
        diag.message.to_string()
    }
}

/// Read and strictly parse a `.cook` file, collecting Cooklang syntax errors/warnings,
/// metadata keys, and ingredient/cookware names for `cook-docs doctor --check-recipes`.
pub fn validate_recipe_file(recipe_path: &str) -> Result<RecipeValidation, String> {
    let content =
        fs::read_to_string(recipe_path).map_err(|e| format!("Failed to read recipe file: {e}"))?;

    let parser =
        cooklang::CooklangParser::new(cooklang::Extensions::all(), cooklang::Converter::default());
    let parsed = parser.parse(&content);
    let report = parsed.report();

    let errors: Vec<String> = report
        .errors()
        .map(|d| format_cooklang_diag(&content, d))
        .collect();
    let warnings: Vec<String> = report
        .warnings()
        .map(|d| format_cooklang_diag(&content, d))
        .collect();

    let mut metadata = std::collections::HashMap::new();
    let mut ingredients = Vec::new();
    let mut cookware = Vec::new();

    if let Some(recipe) = parsed.into_output() {
        for (k, v) in &recipe.metadata.map {
            let key_str = if let Some(s) = k.as_str() {
                s.to_string()
            } else {
                serde_json::to_string(k).unwrap_or_default()
            };
            let val_str = if let Some(s) = v.as_str() {
                s.to_string()
            } else if let Some(b) = v.as_bool() {
                b.to_string()
            } else if let Some(n) = v.as_f64() {
                n.to_string()
            } else if let Some(n) = v.as_i64() {
                n.to_string()
            } else {
                serde_json::to_string(v).unwrap_or_default()
            };
            metadata.insert(key_str, val_str);
        }

        for ing in &recipe.ingredients {
            if !ing.name.trim().is_empty() {
                ingredients.push(ing.name.clone());
            }
        }
        for cw in &recipe.cookware {
            if !cw.name.trim().is_empty() {
                cookware.push(cw.name.clone());
            }
        }
    }

    Ok(RecipeValidation {
        errors,
        warnings,
        metadata,
        ingredients,
        cookware,
    })
}

fn format_cooklang_number(num: &cooklang::quantity::Number) -> (f64, String) {
    match num {
        cooklang::quantity::Number::Regular(f) => {
            let raw = if (*f - f.round()).abs() < 1e-6 {
                format!("{}", f.round() as i64)
            } else {
                let s = format!("{:.2}", f);
                s.trim_end_matches('0').trim_end_matches('.').to_string()
            };
            (*f, raw)
        }
        cooklang::quantity::Number::Fraction {
            whole, num, den, ..
        } => {
            let val = *whole as f64 + (*num as f64 / *den as f64);
            let raw = if *num == 0 {
                whole.to_string()
            } else if *whole == 0 {
                format!("{num}/{den}")
            } else {
                format!("{whole} {num}/{den}")
            };
            (val, raw)
        }
    }
}

fn cooklang_quantity_to_amount(
    qty: &cooklang::quantity::Quantity,
) -> crate::types::IngredientAmount {
    let (is_numeric, quantity, quantity_raw) = match qty.value() {
        cooklang::quantity::Value::Number(num) => {
            let (v, raw) = format_cooklang_number(num);
            (true, v, raw)
        }
        cooklang::quantity::Value::Range { start, end } => {
            let (s_val, s_raw) = format_cooklang_number(start);
            let (_, e_raw) = format_cooklang_number(end);
            (true, s_val, format!("{s_raw}-{e_raw}"))
        }
        cooklang::quantity::Value::Text(t) => (false, 0.0, t.clone()),
    };
    crate::types::IngredientAmount {
        is_numeric,
        quantity,
        quantity_raw,
        unit: qty.unit().unwrap_or("").to_string(),
    }
}

fn aggregate_ingredients(
    parsed_recipe: &cooklang::Recipe,
    converter: &cooklang::Converter,
) -> Vec<crate::types::Ingredient> {
    let mut order: Vec<String> = Vec::new();
    let mut display_names: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut grouped: std::collections::HashMap<String, cooklang::quantity::GroupedQuantity> =
        std::collections::HashMap::new();

    for ing in &parsed_recipe.ingredients {
        if ing.name.trim().is_empty() {
            continue;
        }
        let key = ing.name.trim().to_ascii_lowercase();
        if !display_names.contains_key(&key) {
            order.push(key.clone());
            display_names.insert(key.clone(), ing.name.clone());
            grouped.insert(key.clone(), cooklang::quantity::GroupedQuantity::empty());
        }
        if let Some(qty) = &ing.quantity
            && let Some(group) = grouped.get_mut(&key)
        {
            if let Some(u) = qty.unit()
                && converter.find_unit(u).is_none()
            {
                let lower = u.trim().to_ascii_lowercase();
                let trimmed = lower.trim_end_matches('.');
                if converter.find_unit(&lower).is_some() {
                    let norm_qty =
                        cooklang::quantity::Quantity::new(qty.value().clone(), Some(lower));
                    group.add(&norm_qty, converter);
                    continue;
                } else if trimmed != lower && converter.find_unit(trimmed).is_some() {
                    let norm_qty = cooklang::quantity::Quantity::new(
                        qty.value().clone(),
                        Some(trimmed.to_string()),
                    );
                    group.add(&norm_qty, converter);
                    continue;
                }
            }
            group.add(qty, converter);
        }
    }

    let mut out = Vec::with_capacity(order.len());
    for key in order {
        let name = display_names.remove(&key).unwrap_or_default();
        let group = grouped.remove(&key).unwrap_or_default();
        if group.is_empty() {
            out.push(crate::types::Ingredient {
                name,
                amount: crate::types::IngredientAmount {
                    is_numeric: false,
                    quantity: 0.0,
                    quantity_raw: String::new(),
                    unit: String::new(),
                },
            });
        } else {
            for qty in group.into_vec() {
                out.push(crate::types::Ingredient {
                    name: name.clone(),
                    amount: cooklang_quantity_to_amount(&qty),
                });
            }
        }
    }
    out
}

fn aggregate_cookware(parsed_recipe: &cooklang::Recipe) -> Vec<crate::types::Cookware> {
    let mut order: Vec<String> = Vec::new();
    let mut items: std::collections::HashMap<String, crate::types::Cookware> =
        std::collections::HashMap::new();

    for cw in &parsed_recipe.cookware {
        if cw.name.trim().is_empty() {
            continue;
        }
        let key = cw.name.trim().to_ascii_lowercase();
        let (is_numeric, quantity, quantity_raw) = if let Some(qty) = &cw.quantity {
            let amt = cooklang_quantity_to_amount(qty);
            (amt.is_numeric, amt.quantity, amt.quantity_raw)
        } else {
            (false, 1.0, String::new())
        };

        if let Some(existing) = items.get_mut(&key) {
            if is_numeric && (!existing.is_numeric || quantity > existing.quantity) {
                existing.is_numeric = true;
                existing.quantity = quantity;
                existing.quantity_raw = quantity_raw;
            } else if !existing.is_numeric
                && existing.quantity_raw.is_empty()
                && !quantity_raw.is_empty()
            {
                existing.quantity_raw = quantity_raw;
            }
        } else {
            order.push(key.clone());
            items.insert(
                key,
                crate::types::Cookware {
                    is_numeric,
                    name: cw.name.clone(),
                    quantity,
                    quantity_raw,
                },
            );
        }
    }

    order.into_iter().filter_map(|k| items.remove(&k)).collect()
}

pub fn parse_file(
    recipe_path: &str,
    config: &crate::types::Config,
) -> Result<crate::types::Recipe, String> {
    let recipe_name = get_recipe_name(recipe_path);
    let recipe_slug = slugify_filename(&recipe_name);
    let categories: Vec<String> = relative_recipe_dir(recipe_path, config)
        .unwrap_or_default()
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().to_string()),
            _ => None,
        })
        .collect();
    let rel_root = if categories.is_empty() {
        ".".to_string()
    } else {
        vec![".."; categories.len()].join("/")
    };

    let mut info = crate::types::Info {
        recipe_file_path: recipe_path.to_string(),
        recipe_name,
        recipe_slug,
        new_recipe_file_path: "".to_string(),
        image_file_path: "".to_string(),
        image_file_name: "".to_string(),
        image_slug: "".to_string(),
        rel_root,
        categories,
    };

    info.new_recipe_file_path = get_new_recipe_path(recipe_path, &info.recipe_name, config)
        .to_string_lossy()
        .to_string();

    if let Some(img_path) = get_image_path(recipe_path, &info.recipe_name) {
        info.image_file_path = img_path.to_string_lossy().to_string();
        info.image_file_name = img_path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("")
            .to_string();
        info.image_slug = build_image_slug(&info.image_file_name, &config.image_ext);
    }

    let content = fs::read_to_string(recipe_path)
        .map_err(|e| format!("Failed to read recipe file: {}", e))?;

    let converter = cooklang::Converter::default();
    let parser = cooklang::CooklangParser::new(cooklang::Extensions::all(), converter.clone());
    let parsed_recipe = parser
        .parse(&content)
        .into_output()
        .ok_or_else(|| "Failed to parse recipe content".to_string())?;

    let mut metadata = std::collections::HashMap::new();
    let mut frontmatter = BTreeMap::new();
    for (k, v) in &parsed_recipe.metadata.map {
        let key_str = if let Some(s) = k.as_str() {
            s.to_string()
        } else {
            serde_json::to_string(k).unwrap_or_default()
        };
        let val_str = if let Some(s) = v.as_str() {
            s.to_string()
        } else if let Some(b) = v.as_bool() {
            b.to_string()
        } else if let Some(n) = v.as_f64() {
            n.to_string()
        } else if let Some(n) = v.as_i64() {
            n.to_string()
        } else {
            serde_json::to_string(v).unwrap_or_default()
        };
        if let Ok(json_val) = serde_json::to_value(v) {
            frontmatter.insert(key_str.clone(), json_val);
        }
        metadata.insert(key_str, val_str);
    }

    let mut steps = Vec::new();
    for section in &parsed_recipe.sections {
        for item in &section.content {
            if let cooklang::model::Content::Step(step) = item {
                let mut directions = String::new();
                let mut timers = Vec::new();
                let mut ingredients = Vec::new();
                let mut cookware = Vec::new();

                for step_item in &step.items {
                    match step_item {
                        cooklang::model::Item::Text { value } => {
                            directions.push_str(value);
                        }
                        cooklang::model::Item::Ingredient { index } => {
                            if let Some(ing) = parsed_recipe.ingredients.get(*index) {
                                directions.push_str(&ing.name);

                                let (is_numeric, quantity, quantity_raw, unit) = if let Some(qty) =
                                    &ing.quantity
                                {
                                    let val = match qty.value() {
                                        cooklang::quantity::Value::Number(num) => {
                                            let val = match num {
                                                cooklang::quantity::Number::Regular(f) => *f,
                                                cooklang::quantity::Number::Fraction {
                                                    whole,
                                                    num,
                                                    den,
                                                    ..
                                                } => *whole as f64 + (*num as f64 / *den as f64),
                                            };
                                            (
                                                true,
                                                val,
                                                match num {
                                                    cooklang::quantity::Number::Regular(f) => {
                                                        f.to_string()
                                                    }
                                                    cooklang::quantity::Number::Fraction {
                                                        whole,
                                                        num,
                                                        den,
                                                        ..
                                                    } => {
                                                        if *whole == 0 {
                                                            format!("{}/{}", num, den)
                                                        } else {
                                                            format!("{} {}/{}", whole, num, den)
                                                        }
                                                    }
                                                },
                                            )
                                        }
                                        cooklang::quantity::Value::Range { start, end } => {
                                            let s_val = match start {
                                                cooklang::quantity::Number::Regular(f) => *f,
                                                cooklang::quantity::Number::Fraction {
                                                    whole,
                                                    num,
                                                    den,
                                                    ..
                                                } => *whole as f64 + (*num as f64 / *den as f64),
                                            };
                                            let s_raw = match start {
                                                cooklang::quantity::Number::Regular(f) => {
                                                    f.to_string()
                                                }
                                                cooklang::quantity::Number::Fraction {
                                                    whole,
                                                    num,
                                                    den,
                                                    ..
                                                } => {
                                                    if *whole == 0 {
                                                        format!("{}/{}", num, den)
                                                    } else {
                                                        format!("{} {}/{}", whole, num, den)
                                                    }
                                                }
                                            };
                                            let e_raw = match end {
                                                cooklang::quantity::Number::Regular(f) => {
                                                    f.to_string()
                                                }
                                                cooklang::quantity::Number::Fraction {
                                                    whole,
                                                    num,
                                                    den,
                                                    ..
                                                } => {
                                                    if *whole == 0 {
                                                        format!("{}/{}", num, den)
                                                    } else {
                                                        format!("{} {}/{}", whole, num, den)
                                                    }
                                                }
                                            };
                                            (true, s_val, format!("{}-{}", s_raw, e_raw))
                                        }
                                        cooklang::quantity::Value::Text(t) => {
                                            (false, 0.0, t.clone())
                                        }
                                    };
                                    let u = qty.unit().unwrap_or("").to_string();
                                    (val.0, val.1, val.2, u)
                                } else {
                                    (false, 0.0, String::new(), String::new())
                                };

                                ingredients.push(crate::types::Ingredient {
                                    name: ing.name.clone(),
                                    amount: crate::types::IngredientAmount {
                                        is_numeric,
                                        quantity,
                                        quantity_raw,
                                        unit,
                                    },
                                });
                            }
                        }
                        cooklang::model::Item::Cookware { index } => {
                            if let Some(cw) = parsed_recipe.cookware.get(*index) {
                                directions.push_str(&cw.name);

                                let (is_numeric, quantity, quantity_raw) = if let Some(qty) =
                                    &cw.quantity
                                {
                                    let val = match qty.value() {
                                        cooklang::quantity::Value::Number(num) => {
                                            let val = match num {
                                                cooklang::quantity::Number::Regular(f) => *f,
                                                cooklang::quantity::Number::Fraction {
                                                    whole,
                                                    num,
                                                    den,
                                                    ..
                                                } => *whole as f64 + (*num as f64 / *den as f64),
                                            };
                                            (
                                                true,
                                                val,
                                                match num {
                                                    cooklang::quantity::Number::Regular(f) => {
                                                        f.to_string()
                                                    }
                                                    cooklang::quantity::Number::Fraction {
                                                        whole,
                                                        num,
                                                        den,
                                                        ..
                                                    } => {
                                                        if *whole == 0 {
                                                            format!("{}/{}", num, den)
                                                        } else {
                                                            format!("{} {}/{}", whole, num, den)
                                                        }
                                                    }
                                                },
                                            )
                                        }
                                        cooklang::quantity::Value::Range { start, end } => {
                                            let s_val = match start {
                                                cooklang::quantity::Number::Regular(f) => *f,
                                                cooklang::quantity::Number::Fraction {
                                                    whole,
                                                    num,
                                                    den,
                                                    ..
                                                } => *whole as f64 + (*num as f64 / *den as f64),
                                            };
                                            let s_raw = match start {
                                                cooklang::quantity::Number::Regular(f) => {
                                                    f.to_string()
                                                }
                                                cooklang::quantity::Number::Fraction {
                                                    whole,
                                                    num,
                                                    den,
                                                    ..
                                                } => {
                                                    if *whole == 0 {
                                                        format!("{}/{}", num, den)
                                                    } else {
                                                        format!("{} {}/{}", whole, num, den)
                                                    }
                                                }
                                            };
                                            let e_raw = match end {
                                                cooklang::quantity::Number::Regular(f) => {
                                                    f.to_string()
                                                }
                                                cooklang::quantity::Number::Fraction {
                                                    whole,
                                                    num,
                                                    den,
                                                    ..
                                                } => {
                                                    if *whole == 0 {
                                                        format!("{}/{}", num, den)
                                                    } else {
                                                        format!("{} {}/{}", whole, num, den)
                                                    }
                                                }
                                            };
                                            (true, s_val, format!("{}-{}", s_raw, e_raw))
                                        }
                                        cooklang::quantity::Value::Text(t) => {
                                            (false, 0.0, t.clone())
                                        }
                                    };
                                    (val.0, val.1, val.2)
                                } else {
                                    (false, 1.0, String::new())
                                };

                                cookware.push(crate::types::Cookware {
                                    is_numeric,
                                    name: cw.name.clone(),
                                    quantity,
                                    quantity_raw,
                                });
                            }
                        }
                        cooklang::model::Item::Timer { index } => {
                            if let Some(t) = parsed_recipe.timers.get(*index) {
                                let name = t.name.clone().unwrap_or_default();
                                let (duration, unit) = if let Some(qty) = &t.quantity {
                                    let val = match qty.value() {
                                        cooklang::quantity::Value::Number(num) => match num {
                                            cooklang::quantity::Number::Regular(f) => *f,
                                            cooklang::quantity::Number::Fraction {
                                                whole,
                                                num,
                                                den,
                                                ..
                                            } => *whole as f64 + (*num as f64 / *den as f64),
                                        },
                                        _ => 0.0,
                                    };
                                    let u = qty.unit().unwrap_or("").to_string();
                                    (val, u)
                                } else {
                                    (0.0, String::new())
                                };

                                directions.push_str(&format!("{} {}", duration, unit));
                                timers.push(crate::types::Timer {
                                    name,
                                    duration,
                                    unit,
                                });
                            }
                        }
                        cooklang::model::Item::InlineQuantity { index } => {
                            if let Some(qty) = parsed_recipe.inline_quantities.get(*index) {
                                let duration = match qty.value() {
                                    cooklang::quantity::Value::Number(num) => match num {
                                        cooklang::quantity::Number::Regular(f) => *f,
                                        cooklang::quantity::Number::Fraction {
                                            whole,
                                            num,
                                            den,
                                            ..
                                        } => *whole as f64 + (*num as f64 / *den as f64),
                                    },
                                    _ => 0.0,
                                };
                                let unit = qty.unit().unwrap_or("");
                                directions.push_str(&format!("{} {}", duration, unit));
                            }
                        }
                    }
                }

                steps.push(crate::types::Step {
                    directions,
                    timers,
                    ingredients,
                    cookware,
                    comments: Vec::new(),
                });
            }
        }
    }

    let aggregated_ingredients = aggregate_ingredients(&parsed_recipe, &converter);
    let aggregated_cookware = aggregate_cookware(&parsed_recipe);

    Ok(crate::types::Recipe {
        steps,
        ingredients: aggregated_ingredients,
        cookware: aggregated_cookware,
        metadata,
        frontmatter,
        config: config.clone(),
        info,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Config;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(prefix: &str) -> Self {
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!(
                "cook_docs_cook_{}_{}_{}",
                prefix,
                std::process::id(),
                id
            ));
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn default_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_find_git_repository_root_succeeds() {
        assert!(find_git_repository_root().is_some());
    }

    #[test]
    fn test_path_and_name_helpers() {
        assert!(is_relative_path("./templates/a.gotmpl"));
        assert!(is_relative_path("../a.gotmpl"));
        assert!(!is_relative_path("recipe.md.gotmpl"));
        assert!(!is_relative_path("/abs/recipe.md.gotmpl"));

        assert!(is_base_filename("recipe.md.gotmpl"));
        assert!(!is_base_filename("./recipe.md.gotmpl"));
        assert!(!is_base_filename("dir/recipe.md.gotmpl"));

        assert_eq!(get_recipe_name("a/b/Tomato Soup.cook"), "Tomato Soup");
        assert_eq!(get_recipe_name(""), "");
    }

    #[test]
    fn test_get_new_recipe_path_variants() {
        let mut cfg = default_config();
        assert_eq!(
            get_new_recipe_path("recipes/Tomato Soup.cook", "Tomato Soup", &cfg),
            PathBuf::from("recipes/tomato-soup.md")
        );

        let root = TempDir::new("out_path");
        let sub = root.0.join("breakfast");
        fs::create_dir_all(&sub).unwrap();
        let recipe_file = sub.join("Pancakes.cook");
        fs::write(&recipe_file, "").unwrap();

        cfg.recipe_search_root = root.0.to_string_lossy().to_string();
        cfg.output_dir = root.0.join("out").to_string_lossy().to_string();
        assert_eq!(
            get_new_recipe_path(&recipe_file.to_string_lossy(), "Pancakes", &cfg),
            root.0.join("out/breakfast/pancakes.md")
        );

        cfg.recipe_search_root = "".to_string();
        assert_eq!(
            get_new_recipe_path("/unrelated/dir/Bread.cook", "Bread", &cfg),
            root.0.join("out/bread.md")
        );
    }

    #[test]
    fn test_get_image_path_priority_and_missing() {
        let dir = TempDir::new("images");
        let cook = dir.0.join("Soup.cook");
        fs::write(&cook, "").unwrap();
        let cook_str = cook.to_string_lossy().to_string();

        assert_eq!(get_image_path(&cook_str, "Soup"), None);

        let png = dir.0.join("Soup.png");
        fs::write(&png, b"png").unwrap();
        assert_eq!(get_image_path(&cook_str, "Soup"), Some(png));

        let jpg = dir.0.join("Soup.jpg");
        fs::write(&jpg, b"jpg").unwrap();
        assert_eq!(get_image_path(&cook_str, "Soup"), Some(jpg));
    }

    #[test]
    fn test_find_recipe_file_paths_respects_ignore_and_git() {
        let dir = TempDir::new("find");
        fs::create_dir_all(dir.0.join("keep")).unwrap();
        fs::create_dir_all(dir.0.join("skip")).unwrap();
        fs::create_dir_all(dir.0.join(".git")).unwrap();

        fs::write(dir.0.join("keep/One.cook"), "").unwrap();
        fs::write(dir.0.join("keep/note.txt"), "").unwrap();
        fs::write(dir.0.join("skip/Two.cook"), "").unwrap();
        fs::write(dir.0.join(".git/Three.cook"), "").unwrap();

        let paths = find_recipe_file_paths(&dir.0.to_string_lossy(), "");
        assert!(paths.iter().any(|p| p.ends_with("keep/One.cook")));
        assert!(paths.iter().any(|p| p.ends_with("skip/Two.cook")));
        assert!(!paths.iter().any(|p| p.ends_with(".git/Three.cook")));

        let repo_paths = find_recipe_file_paths("testdata", ".cookdocsignore");
        assert!(
            repo_paths
                .iter()
                .any(|p| p.ends_with("My Test Recipe.cook"))
        );
    }

    #[test]
    fn test_parse_file_missing_returns_error() {
        let cfg = default_config();
        let err = parse_file("/nonexistent/path/missing.cook", &cfg).unwrap_err();
        assert!(err.contains("Failed to read recipe file"));
    }

    #[test]
    fn test_aggregate_ingredients_and_cookware_across_steps() {
        let dir = TempDir::new("aggregate");
        let recipe_path = dir.0.join("Layered Cake.cook");
        fs::write(
            &recipe_path,
            "\
Whisk @all-purpose flour{2%cups}, @granulated sugar{1%cup}, @cinnamon{1%Tbsp}, and @salt in #mixing bowl{1}.

Make topping with @All-Purpose Flour{1/2%cup}, @granulated sugar{1/2%cup}, @cinnamon{3%tsp}, @lemon{2}, and @lemon{1%tbsp} in #mixing bowl{2} and #mixing bowl{}.
",
        )
        .unwrap();

        let cfg = default_config();
        let parsed = parse_file(&recipe_path.to_string_lossy(), &cfg).unwrap();

        // Per-step lists preserved for backwards compatibility
        assert_eq!(parsed.steps.len(), 2);
        assert_eq!(parsed.steps[0].ingredients.len(), 4);
        assert_eq!(parsed.steps[1].ingredients.len(), 5);

        // Top-level aggregated ingredients combine repeated items across steps
        let flour = parsed
            .ingredients
            .iter()
            .find(|i| i.name == "all-purpose flour")
            .unwrap();
        assert!((flour.amount.quantity - 2.5).abs() < 1e-6);
        assert_eq!(flour.amount.unit, "cups");
        assert_eq!(
            parsed
                .ingredients
                .iter()
                .filter(|i| i.name.eq_ignore_ascii_case("all-purpose flour"))
                .count(),
            1
        );

        let sugar = parsed
            .ingredients
            .iter()
            .find(|i| i.name == "granulated sugar")
            .unwrap();
        assert!((sugar.amount.quantity - 1.5).abs() < 1e-6);

        // 1 tbsp + 3 tsp = 2 tbsp
        let cinnamon = parsed
            .ingredients
            .iter()
            .find(|i| i.name == "cinnamon")
            .unwrap();
        assert!((cinnamon.amount.quantity - 2.0).abs() < 1e-3);
        assert_eq!(cinnamon.amount.unit, "tbsp");

        // Bare ingredient preserved with quantity 0
        let salt = parsed
            .ingredients
            .iter()
            .find(|i| i.name == "salt")
            .unwrap();
        assert!(!salt.amount.is_numeric);
        assert_eq!(salt.amount.quantity, 0.0);

        // Incompatible units (2 whole lemons + 1 tbsp lemon) kept as separate aggregated entries
        let lemons: Vec<&crate::types::Ingredient> = parsed
            .ingredients
            .iter()
            .filter(|i| i.name == "lemon")
            .collect();
        assert_eq!(lemons.len(), 2);

        // Cookware deduplicated with max explicit quantity
        assert_eq!(parsed.cookware.len(), 1);
        assert_eq!(parsed.cookware[0].name, "mixing bowl");
        assert_eq!(parsed.cookware[0].quantity, 2.0);
    }

    #[test]
    fn test_parse_file_quantities_cookware_timers_metadata() {
        let dir = TempDir::new("parse");
        let recipe_path = dir.0.join("Rich Recipe.cook");
        let img_path = dir.0.join("Rich Recipe.jpg");
        fs::write(&img_path, b"img").unwrap();
        fs::write(
            &recipe_path,
            "\
>> servings: 4
>> flag: true
>> ratio: 1.5
>> count: 2
Add @flour{1/2%cup}, @sugar{1 1/2%tbsp}, @butter{1-2%sticks}, @salt{some}, and @pepper in #bowl{1}, #pan{1/2}, #tray{1-2}, #pot{big}, and #spoon for ~boil{1/2%hour} and ~rest{10%minutes} at ~{20%minutes}.
",
        )
        .unwrap();

        let cfg = default_config();
        let parsed = parse_file(&recipe_path.to_string_lossy(), &cfg).unwrap();
        assert_eq!(parsed.info.recipe_name, "Rich Recipe");
        assert_eq!(parsed.info.image_file_name, "Rich Recipe.jpg");
        assert_eq!(parsed.metadata.get("servings"), Some(&"4".to_string()));
        assert_eq!(parsed.metadata.get("flag"), Some(&"true".to_string()));
        assert_eq!(parsed.metadata.get("ratio"), Some(&"1.5".to_string()));
        assert_eq!(parsed.metadata.get("count"), Some(&"2".to_string()));
        assert_eq!(parsed.steps.len(), 1);

        let step = &parsed.steps[0];
        assert_eq!(step.ingredients.len(), 5);
        assert_eq!(step.ingredients[0].amount.quantity_raw, "1/2");
        assert!((step.ingredients[0].amount.quantity - 0.5).abs() < 1e-6);
        assert_eq!(step.ingredients[1].amount.quantity_raw, "1 1/2");
        assert!((step.ingredients[1].amount.quantity - 1.5).abs() < 1e-6);
        assert_eq!(step.ingredients[2].amount.quantity_raw, "1-2");
        assert!(!step.ingredients[3].amount.is_numeric);
        assert_eq!(step.ingredients[3].amount.quantity_raw, "some");
        assert!(!step.ingredients[4].amount.is_numeric);

        assert_eq!(step.cookware.len(), 5);
        assert_eq!(step.cookware[1].quantity_raw, "1/2");
        assert_eq!(step.cookware[2].quantity_raw, "1-2");
        assert_eq!(step.cookware[3].quantity_raw, "big");
        assert_eq!(step.cookware[4].quantity, 1.0);

        assert_eq!(step.timers.len(), 3);
        assert_eq!(step.timers[0].name, "boil");
        assert!((step.timers[0].duration - 0.5).abs() < 1e-6);
        assert_eq!(step.timers[0].unit, "hour");
    }

    #[test]
    fn test_parse_file_yaml_frontmatter_fraction_ranges_and_inline_quantities() {
        let dir = TempDir::new("yaml_frac");
        let recipe_path = dir.0.join("Advanced.cook");
        fs::write(
            &recipe_path,
            "\
---
title: Advanced Dish
vegan: true
rating: 4.5
offset: -3
tags:
  - quick
  - dinner
123: numeric-key
---
Mix @milk{1/2-3/4%cup} and @cream{1 1/2-2 1/4%cup} in #bowl{1 1/2}, #pan{1/2-3/4}, and #pot{1 1/2-2 1/4}.

Bake at 180°C for ~{10-15%minutes}.
",
        )
        .unwrap();

        let cfg = default_config();
        let parsed = parse_file(&recipe_path.to_string_lossy(), &cfg).unwrap();
        assert_eq!(
            parsed.metadata.get("title"),
            Some(&"Advanced Dish".to_string())
        );
        assert_eq!(parsed.metadata.get("vegan"), Some(&"true".to_string()));
        assert_eq!(parsed.metadata.get("rating"), Some(&"4.5".to_string()));
        assert_eq!(parsed.metadata.get("offset"), Some(&"-3".to_string()));
        assert!(
            parsed
                .metadata
                .get("tags")
                .is_some_and(|v| v.contains("quick") && v.contains("dinner"))
        );
        assert_eq!(parsed.metadata.get("123"), Some(&"numeric-key".to_string()));

        assert_eq!(parsed.steps.len(), 2);
        let s0 = &parsed.steps[0];
        assert_eq!(s0.ingredients[0].amount.quantity_raw, "1/2-3/4");
        assert!((s0.ingredients[0].amount.quantity - 0.5).abs() < 1e-6);
        assert_eq!(s0.ingredients[1].amount.quantity_raw, "1 1/2-2 1/4");
        assert!((s0.ingredients[1].amount.quantity - 1.5).abs() < 1e-6);

        assert_eq!(s0.cookware[0].quantity_raw, "1 1/2");
        assert!((s0.cookware[0].quantity - 1.5).abs() < 1e-6);
        assert_eq!(s0.cookware[1].quantity_raw, "1/2-3/4");
        assert!((s0.cookware[1].quantity - 0.5).abs() < 1e-6);
        assert_eq!(s0.cookware[2].quantity_raw, "1 1/2-2 1/4");
        assert!((s0.cookware[2].quantity - 1.5).abs() < 1e-6);

        let s1 = &parsed.steps[1];
        assert!(
            s1.directions.contains("180 °C"),
            "expected inline temperature in directions: {}",
            s1.directions
        );
        assert_eq!(s1.timers.len(), 1);
        assert_eq!(s1.timers[0].duration, 0.0);
        assert_eq!(s1.timers[0].unit, "minutes");

        // Invalid timer syntax triggers Cooklang parse error
        let invalid_path = dir.0.join("Invalid.cook");
        fs::write(&invalid_path, "Wait for ~rest{}.").unwrap();
        let err = parse_file(&invalid_path.to_string_lossy(), &cfg).unwrap_err();
        assert!(err.contains("Failed to parse recipe content"));
    }

    #[test]
    fn test_find_recipe_file_paths_invalid_ignore_and_path_fallbacks() {
        let dir = TempDir::new("bad_ignore");
        fs::write(dir.0.join("One.cook"), "Boil @water{1%l}.").unwrap();
        let bad_ignore = dir.0.join(".badignore");
        fs::write(&bad_ignore, b"\xff\xfe\n").unwrap();

        // Absolute ignore_file path overrides git root join and exercises error warning path
        let found = find_recipe_file_paths(&dir.0.to_string_lossy(), &bad_ignore.to_string_lossy());
        assert!(found.iter().any(|p| p.ends_with("One.cook")));

        // Bare filename & nonexistent root fallbacks in get_new_recipe_path / get_image_path
        let mut cfg = default_config();
        assert_eq!(
            get_new_recipe_path("Soup.cook", "Soup", &cfg),
            PathBuf::from("soup.md")
        );
        cfg.recipe_search_root = "/nonexistent_cook_docs_root_123".to_string();
        cfg.output_dir = "/tmp/out".to_string();
        assert_eq!(
            get_new_recipe_path(
                "/nonexistent_cook_docs_root_123/sub/Soup.cook",
                "Soup",
                &cfg
            ),
            PathBuf::from("/tmp/out/sub/soup.md")
        );
        assert_eq!(get_image_path("/", "Missing"), None);
    }

    #[test]
    fn test_image_discovery_webp_jpeg_and_slug_builder() {
        let dir = TempDir::new("img_slug");
        let cook_path = dir.0.join("My Dish.cook");
        fs::write(&cook_path, "Season with @salt{}.").unwrap();

        assert_eq!(build_image_slug("", ""), "");
        assert_eq!(build_image_slug("My Dish.JPG", ""), "my-dish.jpg");
        assert_eq!(build_image_slug("My Dish.jpg", ".WEBP"), "my-dish.webp");

        // .png only
        fs::write(dir.0.join("My Dish.png"), b"p").unwrap();
        assert!(
            get_image_path(&cook_path.to_string_lossy(), "My Dish")
                .unwrap()
                .ends_with("My Dish.png")
        );

        // .jpeg beats .png
        fs::write(dir.0.join("My Dish.jpeg"), b"j").unwrap();
        assert!(
            get_image_path(&cook_path.to_string_lossy(), "My Dish")
                .unwrap()
                .ends_with("My Dish.jpeg")
        );

        // .jpg beats .jpeg
        fs::write(dir.0.join("My Dish.jpg"), b"j").unwrap();
        assert!(
            get_image_path(&cook_path.to_string_lossy(), "My Dish")
                .unwrap()
                .ends_with("My Dish.jpg")
        );

        // .webp beats .jpg
        fs::write(dir.0.join("My Dish.webp"), b"w").unwrap();
        assert!(
            get_image_path(&cook_path.to_string_lossy(), "My Dish")
                .unwrap()
                .ends_with("My Dish.webp")
        );

        let mut cfg = default_config();
        cfg.image_ext = "png".to_string();
        let parsed = parse_file(&cook_path.to_string_lossy(), &cfg).unwrap();
        assert_eq!(parsed.info.image_file_name, "My Dish.webp");
        assert_eq!(parsed.info.image_slug, "my-dish.png");
        assert_eq!(parsed.steps[0].ingredients[0].amount.quantity, 0.0);
        assert!(!parsed.steps[0].ingredients[0].amount.is_numeric);
    }

    #[test]
    fn test_resolve_recipe_paths_files_dirs_dedupe_and_errors() {
        let dir = TempDir::new("resolve_paths");
        let breads = dir.0.join("breads");
        let soups = dir.0.join("soups");
        fs::create_dir_all(&breads).unwrap();
        fs::create_dir_all(&soups).unwrap();
        let bagels = breads.join("Bagels.cook");
        let chili = soups.join("Chili.cook");
        let notes = breads.join("notes.txt");
        fs::write(&bagels, "Bake @flour{500%g}.").unwrap();
        fs::write(&chili, "Simmer @beans{1%can}.").unwrap();
        fs::write(&notes, "plain text").unwrap();

        let mut cfg = default_config();
        cfg.recipe_search_root = dir.0.to_string_lossy().to_string();
        cfg.ignore_file = String::new();

        // Duplicate entries (absolute + relative to search_root + subdir) deduped
        cfg.recipe_files = vec![
            bagels.to_string_lossy().to_string(),
            "breads/Bagels.cook".to_string(),
            "soups".to_string(),
        ];
        let resolved = resolve_recipe_paths(&dir.0, &cfg).unwrap();
        assert_eq!(resolved.len(), 2);

        // Non-.cook file rejected
        cfg.recipe_files = vec!["breads/notes.txt".to_string()];
        assert!(
            resolve_recipe_paths(&dir.0, &cfg)
                .unwrap_err()
                .contains(".cook")
        );

        // Missing file rejected
        cfg.recipe_files = vec!["breads/Missing.cook".to_string()];
        assert!(
            resolve_recipe_paths(&dir.0, &cfg)
                .unwrap_err()
                .contains("not found")
        );
    }
}

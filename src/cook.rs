use ignore::gitignore::GitignoreBuilder;
use log::{debug, warn};
use std::fs;
use std::path::{Path, PathBuf};
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

pub fn get_new_recipe_path(
    recipe_path: &str,
    recipe_name: &str,
    config: &crate::types::Config,
) -> PathBuf {
    let file_name = format!("{}.md", recipe_name.replace(' ', "-").to_lowercase());

    if config.output_dir.is_empty() {
        let parent = Path::new(recipe_path)
            .parent()
            .unwrap_or_else(|| Path::new(""));
        return parent.join(file_name);
    }

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

    match abs_recipe_path.strip_prefix(&abs_search_root) {
        Ok(rel) => {
            let rel_dir = rel.parent().unwrap_or_else(|| Path::new(""));
            Path::new(&config.output_dir).join(rel_dir).join(file_name)
        }
        Err(_) => Path::new(&config.output_dir).join(file_name),
    }
}

pub fn get_image_path(recipe_path: &str, recipe_name: &str) -> Option<PathBuf> {
    let parent = Path::new(recipe_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));

    // Check JPG first
    let jpg_path = parent.join(format!("{}.jpg", recipe_name));
    if jpg_path.exists() {
        return Some(jpg_path);
    }

    // Check PNG second
    let png_path = parent.join(format!("{}.png", recipe_name));
    if png_path.exists() {
        return Some(png_path);
    }

    warn!("Image file {} missing.", parent.join(recipe_name).display());
    None
}

pub fn find_recipe_file_paths(recipe_search_root: &str, ignore_file: &str) -> Vec<PathBuf> {
    let mut recipe_paths = Vec::new();

    // Build gitignore matcher
    let relative_dir = match find_git_repository_root() {
        Some(root) => root,
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

            // Get relative path for ignore checking
            if path
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

    recipe_paths
}

pub fn parse_file(
    recipe_path: &str,
    config: &crate::types::Config,
) -> Result<crate::types::Recipe, String> {
    let mut info = crate::types::Info {
        recipe_file_path: recipe_path.to_string(),
        recipe_name: get_recipe_name(recipe_path),
        new_recipe_file_path: "".to_string(),
        image_file_path: "".to_string(),
        image_file_name: "".to_string(),
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
    }

    let content = fs::read_to_string(recipe_path)
        .map_err(|e| format!("Failed to read recipe file: {}", e))?;

    let parser =
        cooklang::CooklangParser::new(cooklang::Extensions::all(), cooklang::Converter::default());
    let parsed_recipe = parser
        .parse(&content)
        .into_output()
        .ok_or_else(|| "Failed to parse recipe content".to_string())?;

    let mut metadata = std::collections::HashMap::new();
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
                                    (false, 1.0, String::new(), String::new())
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

    Ok(crate::types::Recipe {
        steps,
        metadata,
        config: config.clone(),
        info,
    })
}

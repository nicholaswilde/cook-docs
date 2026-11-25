pub mod config;
pub mod cook;
pub mod doctor;
pub mod document;
pub mod enrichment;
pub mod merge;
pub mod types;
pub mod update;
pub mod watch;

use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Resolve configured `recipe_search_root` against the current working directory when relative.
pub fn resolve_search_root(recipe_search_root: &str) -> PathBuf {
    if Path::new(recipe_search_root).is_absolute() {
        PathBuf::from(recipe_search_root)
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(recipe_search_root)
    }
}

/// Parse and render a single `.cook` recipe file.
pub fn render_recipe(path: &str, config: &types::Config) -> Result<(), String> {
    let resources = Arc::new(enrichment::RenderResources::from_config(config));
    render_recipe_with_resources(path, config, &resources)
}

fn render_recipe_with_resources(
    path: &str,
    config: &types::Config,
    resources: &Arc<enrichment::RenderResources>,
) -> Result<(), String> {
    let link_index = config.link_in_steps.then_some(&resources.links);
    let recipe = cook::parse_file_with_links(path, config, link_index)?;
    document::print_documentation_with_resources(&recipe, resources);
    Ok(())
}

/// Render a batch of recipe paths (serially on `--dry-run`, threaded otherwise).
/// Returns `(rendered_ok, failed)`.
pub fn render_recipes(recipe_paths: &[PathBuf], config: &types::Config) -> (usize, usize) {
    let recipe_paths_strs: Vec<String> = recipe_paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    if config.jsonify {
        let link_index = config
            .link_in_steps
            .then(|| enrichment::RecipeLinkIndex::build(config));
        let mut recipes = Vec::with_capacity(recipe_paths_strs.len());
        let mut ok = 0;
        let mut failed = 0;
        for path_str in recipe_paths_strs {
            match cook::parse_file_with_links(&path_str, config, link_index.as_ref()) {
                Ok(recipe) => {
                    recipes.push(recipe);
                    ok += 1;
                }
                Err(e) => {
                    log::warn!(
                        "Error parsing file for recipe {}, skipping: {}",
                        path_str,
                        e
                    );
                    failed += 1;
                }
            }
        }
        let json_out = if recipes.len() == 1 {
            serde_json::to_string_pretty(&recipes[0])
        } else {
            serde_json::to_string_pretty(&recipes)
        };
        if let Ok(j) = json_out {
            println!("{}", j);
        }
        return (ok, failed);
    }

    let resources = Arc::new(enrichment::RenderResources::from_config(config));

    if config.dry_run {
        let mut ok = 0;
        let mut failed = 0;
        for path_str in recipe_paths_strs {
            if let Err(e) = render_recipe_with_resources(&path_str, config, &resources) {
                log::warn!(
                    "Error parsing file for recipe {}, skipping: {}",
                    path_str,
                    e
                );
                failed += 1;
            } else {
                ok += 1;
            }
        }
        return (ok, failed);
    }

    let mut handles = Vec::with_capacity(recipe_paths_strs.len());
    for path_str in recipe_paths_strs {
        let config_clone = config.clone();
        let resources_clone = Arc::clone(&resources);
        handles.push(std::thread::spawn(move || {
            if let Err(e) = render_recipe_with_resources(&path_str, &config_clone, &resources_clone)
            {
                log::warn!(
                    "Error parsing file for recipe {}, skipping: {}",
                    path_str,
                    e
                );
                false
            } else {
                true
            }
        }));
    }

    let mut ok = 0;
    let mut failed = 0;
    for handle in handles {
        match handle.join() {
            Ok(true) => ok += 1,
            Ok(false) | Err(_) => failed += 1,
        }
    }
    (ok, failed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn test_resolve_search_root_absolute_and_relative() {
        let abs = std::env::temp_dir().join("cook_docs_abs_check");
        assert_eq!(resolve_search_root(&abs.to_string_lossy()), abs);

        let rel = resolve_search_root("testdata");
        assert!(rel.ends_with("testdata"));
        assert!(rel.is_absolute());
    }

    #[test]
    fn test_render_recipes_dry_run_and_threaded_ok_and_failed() {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("cook_docs_lib_{}_{}", std::process::id(), id));
        fs::create_dir_all(&dir).unwrap();

        let valid = dir.join("Soup.cook");
        fs::write(&valid, "Boil @water{1%l} for ~{5%minutes}.").unwrap();
        let missing = dir.join("Missing.cook");

        let mut cfg = types::Config {
            dry_run: true,
            recipe_search_root: dir.to_string_lossy().to_string(),
            ..types::Config::default()
        };

        let paths = vec![valid.clone(), missing.clone()];
        assert_eq!(render_recipes(&paths, &cfg), (1, 1));
        assert!(!dir.join("soup.md").exists());

        cfg.jsonify = true;
        assert_eq!(render_recipes(&paths, &cfg), (1, 1));
        assert!(!dir.join("soup.md").exists());

        cfg.jsonify = false;
        cfg.dry_run = false;
        assert_eq!(render_recipes(&paths, &cfg), (1, 1));
        assert!(dir.join("soup.md").exists());

        let _ = fs::remove_dir_all(&dir);
    }
}

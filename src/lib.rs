pub mod config;
pub mod cook;
pub mod doctor;
pub mod document;
pub mod types;
pub mod update;
pub mod watch;

use std::path::{Path, PathBuf};

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
    let recipe = cook::parse_file(path, config)?;
    document::print_documentation(&recipe);
    Ok(())
}

/// Render a batch of recipe paths (serially on `--dry-run`, threaded otherwise).
/// Returns `(rendered_ok, failed)`.
pub fn render_recipes(recipe_paths: &[PathBuf], config: &types::Config) -> (usize, usize) {
    let recipe_paths_strs: Vec<String> = recipe_paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    if config.dry_run {
        let mut ok = 0;
        let mut failed = 0;
        for path_str in recipe_paths_strs {
            if let Err(e) = render_recipe(&path_str, config) {
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
        handles.push(std::thread::spawn(move || {
            if let Err(e) = render_recipe(&path_str, &config_clone) {
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

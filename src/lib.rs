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
            jsonify: false,
            ignore_file: ".cookdocsignore".to_string(),
            recipe_search_root: dir.to_string_lossy().to_string(),
            log_level: "info".to_string(),
            template_files: vec!["recipe.md.gotmpl".to_string()],
            word_wrap: 120,
            output_dir: String::new(),
        };

        let paths = vec![valid.clone(), missing.clone()];
        assert_eq!(render_recipes(&paths, &cfg), (1, 1));
        assert!(!dir.join("soup.md").exists());

        cfg.dry_run = false;
        assert_eq!(render_recipes(&paths, &cfg), (1, 1));
        assert!(dir.join("soup.md").exists());

        let _ = fs::remove_dir_all(&dir);
    }
}

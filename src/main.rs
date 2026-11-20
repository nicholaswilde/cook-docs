use std::path::{Path, PathBuf};

fn render_recipe(path: &str, config: &cook_docs::types::Config) -> Result<(), String> {
    let recipe = cook_docs::cook::parse_file(path, config)?;
    cook_docs::document::print_documentation(&recipe);
    Ok(())
}

fn main() {
    // 1. Load config and CLI arguments
    let args: Vec<String> = std::env::args().collect();
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let config = match cook_docs::config::load_action(&args_ref) {
        Ok(cook_docs::config::CliAction::Completions(shell)) => {
            cook_docs::config::generate_completions(shell, &mut std::io::stdout());
            return;
        }
        Ok(cook_docs::config::CliAction::Run(cfg)) => cfg,
        Err(e) => {
            eprintln!("Failed to parse config or CLI args: {}", e);
            std::process::exit(1);
        }
    };

    // 2. Initialize logger
    let log_level = match config.log_level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" | "warning" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };
    env_logger::Builder::new().filter_level(log_level).init();

    // 3. Resolve search root
    let search_root = if Path::new(&config.recipe_search_root).is_absolute() {
        PathBuf::from(&config.recipe_search_root)
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(&config.recipe_search_root)
    };

    // 4. Find recipe paths
    let search_root_str = search_root.to_string_lossy();
    let recipe_paths =
        cook_docs::cook::find_recipe_file_paths(&search_root_str, &config.ignore_file);

    let recipe_paths_strs: Vec<String> = recipe_paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    log::info!("Found recipes [{}]", recipe_paths_strs.join(", "));
    log::debug!(
        "Rendering from optional template files [{}]",
        config.template_files.join(", ")
    );

    // 5. Parse and render recipes
    let mut handles = Vec::new();
    for path_str in recipe_paths_strs {
        let config_clone = config.clone();
        if config.dry_run {
            // Serially render dry run
            if let Err(e) = render_recipe(&path_str, &config_clone) {
                log::warn!(
                    "Error parsing file for recipe {}, skipping: {}",
                    path_str,
                    e
                );
            }
        } else {
            let handle = std::thread::spawn(move || {
                if let Err(e) = render_recipe(&path_str, &config_clone) {
                    log::warn!(
                        "Error parsing file for recipe {}, skipping: {}",
                        path_str,
                        e
                    );
                }
            });
            handles.push(handle);
        }
    }

    for h in handles {
        let _ = h.join();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn test_render_recipe_success_and_error() {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("cook_docs_main_{}_{}", std::process::id(), id));
        fs::create_dir_all(&dir).unwrap();
        let recipe = dir.join("Test.cook");
        fs::write(&recipe, "Boil @water{1%l} for ~{5%minutes}.").unwrap();

        let cfg = cook_docs::types::Config {
            dry_run: true,
            jsonify: false,
            ignore_file: ".cookdocsignore".to_string(),
            recipe_search_root: dir.to_string_lossy().to_string(),
            log_level: "info".to_string(),
            template_files: vec!["recipe.md.gotmpl".to_string()],
            word_wrap: 120,
            output_dir: "".to_string(),
        };

        assert!(render_recipe(&recipe.to_string_lossy(), &cfg).is_ok());
        assert!(render_recipe("/nonexistent/Test.cook", &cfg).is_err());

        let _ = fs::remove_dir_all(&dir);
    }
}

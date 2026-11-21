use std::io::IsTerminal;
use std::time::Duration;

fn init_logger(level: &str) {
    let log_level = match level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" | "warning" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };
    env_logger::Builder::new().filter_level(log_level).init();
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
        Ok(cook_docs::config::CliAction::Man) => {
            if let Err(e) = cook_docs::config::generate_manpage(&mut std::io::stdout()) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
            return;
        }
        Ok(cook_docs::config::CliAction::Doctor {
            json,
            config_path,
            config_result,
        }) => {
            let report = cook_docs::doctor::run_doctor(config_path.as_deref(), &config_result);
            if json {
                match serde_json::to_string_pretty(&report) {
                    Ok(json_str) => println!("{}", json_str),
                    Err(e) => {
                        eprintln!("Failed to serialize doctor report: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                let colorize =
                    std::io::stdout().is_terminal() && std::env::var("NO_COLOR").is_err();
                print!(
                    "{}",
                    cook_docs::doctor::format_doctor_report(&report, colorize)
                );
            }
            if !report.healthy {
                std::process::exit(1);
            }
            return;
        }
        Ok(cook_docs::config::CliAction::SelfUpdate {
            check,
            yes,
            version,
        }) => {
            if let Err(e) = cook_docs::update::run_self_update(
                cook_docs::update::SelfUpdateOptions {
                    check,
                    yes,
                    version,
                },
                &mut std::io::stdout(),
            ) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
            return;
        }
        Ok(cook_docs::config::CliAction::Watch {
            config,
            debounce_ms,
        }) => {
            init_logger(&config.log_level);
            if let Err(e) = cook_docs::watch::run_watch(
                &config,
                cook_docs::watch::WatchOptions {
                    debounce: Duration::from_millis(debounce_ms),
                },
            ) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
            return;
        }
        Ok(cook_docs::config::CliAction::Run(cfg)) => cfg,
        Err(e) => {
            eprintln!("Failed to parse config or CLI args: {}", e);
            std::process::exit(1);
        }
    };

    // 2. Initialize logger
    init_logger(&config.log_level);

    // 3. Resolve search root
    let search_root = cook_docs::resolve_search_root(&config.recipe_search_root);

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
    let _ = cook_docs::render_recipes(&recipe_paths, &config);
}

#[cfg(test)]
mod tests {
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

        assert!(cook_docs::render_recipe(&recipe.to_string_lossy(), &cfg).is_ok());
        assert!(cook_docs::render_recipe("/nonexistent/Test.cook", &cfg).is_err());

        let _ = fs::remove_dir_all(&dir);
    }
}

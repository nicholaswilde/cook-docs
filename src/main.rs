use std::io::IsTerminal;
use std::time::Duration;

fn parse_log_level(level: &str) -> log::LevelFilter {
    match level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" | "warning" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    }
}

fn init_logger(level: &str) {
    let log_level = parse_log_level(level);
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
            check_recipes,
            config_path,
            config_result,
        }) => {
            let report = cook_docs::doctor::run_doctor_with_options(
                config_path.as_deref(),
                &config_result,
                cook_docs::doctor::DoctorOptions { check_recipes },
            );
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
    let recipe_paths = match cook_docs::cook::resolve_recipe_paths(&search_root, &config) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

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
            recipe_search_root: dir.to_string_lossy().to_string(),
            ..cook_docs::types::Config::default()
        };

        assert!(cook_docs::render_recipe(&recipe.to_string_lossy(), &cfg).is_ok());
        assert!(cook_docs::render_recipe("/nonexistent/Test.cook", &cfg).is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_log_level_variants() {
        assert_eq!(super::parse_log_level("trace"), log::LevelFilter::Trace);
        assert_eq!(super::parse_log_level("DEBUG"), log::LevelFilter::Debug);
        assert_eq!(super::parse_log_level("info"), log::LevelFilter::Info);
        assert_eq!(super::parse_log_level("warn"), log::LevelFilter::Warn);
        assert_eq!(super::parse_log_level("WARNING"), log::LevelFilter::Warn);
        assert_eq!(super::parse_log_level("error"), log::LevelFilter::Error);
        assert_eq!(super::parse_log_level("unknown"), log::LevelFilter::Info);
    }
}

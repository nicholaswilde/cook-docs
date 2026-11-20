use crate::types::Config;
use clap::{Arg, ArgAction, Command};
use clap_complete::Shell;
use figment::{
    Figment,
    providers::{Format, Yaml},
};
use serde::Deserialize;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug)]
pub enum CliAction {
    Run(Config),
    Completions(Shell),
}

/// Build the clap CLI command definition for `cook-docs`.
pub fn build_cli() -> Command {
    Command::new("cook-docs")
        .about("cook-docs automatically generates markdown documentation for cook recipes from template files")
        .version(env!("CARGO_PKG_VERSION"))
        .disable_version_flag(true)
        .arg(Arg::new("version")
            .short('v')
            .long("version")
            .action(ArgAction::Version)
            .help("version for cook-docs"))
        .arg(Arg::new("dry-run")
            .short('d')
            .long("dry-run")
            .action(ArgAction::SetTrue)
            .help("don't actually render any markdown files just print to stdout passed"))
        .arg(Arg::new("jsonify")
            .short('j')
            .long("jsonify")
            .action(ArgAction::SetTrue)
            .help("parse the recipe and display it in json format"))
        .arg(Arg::new("ignore-file")
            .short('i')
            .long("ignore-file")
            .help("filename to use as an ignore file to exclude recipe directories (default \".cookdocsignore\")"))
        .arg(Arg::new("recipe-search-root")
            .short('c')
            .long("recipe-search-root")
            .help("directory to search recursively within for recipes (default \".\")"))
        .arg(Arg::new("log-level")
            .short('l')
            .long("log-level")
            .help("level of logs that should printed, one of (panic, fatal, error, warning, info, debug, trace) (default \"info\")"))
        .arg(Arg::new("template-files")
            .short('t')
            .long("template-files")
            .help("gotemplate file paths relative to each recipe directory from which documentation will be generated (default \"recipe.md.gotmpl\")"))
        .arg(Arg::new("word-wrap")
            .short('w')
            .long("word-wrap")
            .help("word wrap line length for recipe steps section (default 120)"))
        .arg(Arg::new("output-dir")
            .short('o')
            .long("output-dir")
            .help("custom output directory to write generated markdown documentation"))
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
}

/// Write static shell completions for `shell` to `writer`.
pub fn generate_completions(shell: Shell, writer: &mut dyn Write) {
    let mut cmd = build_cli();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, bin_name, writer);
}

#[derive(Deserialize, Debug)]
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
}

pub fn load_config(args: &[&str]) -> Result<Config, String> {
    match load_action(args)? {
        CliAction::Run(cfg) => Ok(cfg),
        CliAction::Completions(_) => {
            Err("Subcommand 'completions' cannot be used as config".to_string())
        }
    }
}

/// Parse CLI args into an actionable command, loading config files and env vars when running normally.
pub fn load_action(args: &[&str]) -> Result<CliAction, String> {
    let mut config_paths = Vec::new();
    config_paths.push(PathBuf::from("/etc/cook-docs/.cookdocs.yaml"));
    config_paths.push(PathBuf::from("/etc/cook-docs/.cookdocs.yml"));

    if let Ok(home) = std::env::var("HOME") {
        let home_path = PathBuf::from(home);
        config_paths.push(home_path.join(".config/.cookdocs.yaml"));
        config_paths.push(home_path.join(".config/.cookdocs.yml"));
    }

    config_paths.push(PathBuf::from(".cookdocs.yaml"));
    config_paths.push(PathBuf::from(".cookdocs.yml"));

    load_action_with(args, &config_paths, |k| std::env::var(k).ok())
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

    Ok(CliAction::Run(resolve_config(
        &matches,
        config_paths,
        env_get,
    )))
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
        CliAction::Completions(_) => {
            Err("Subcommand 'completions' cannot be used as config".to_string())
        }
    }
}

fn resolve_config<F>(matches: &clap::ArgMatches, config_paths: &[PathBuf], env_get: F) -> Config
where
    F: Fn(&str) -> Option<String>,
{
    // 1. Set default configuration values
    let mut config = Config {
        dry_run: false,
        jsonify: false,
        ignore_file: ".cookdocsignore".to_string(),
        recipe_search_root: ".".to_string(),
        log_level: "info".to_string(),
        template_files: vec!["recipe.md.gotmpl".to_string()],
        word_wrap: 120,
        output_dir: "".to_string(),
    };

    // 2. Resolve the config file path if it exists
    let mut found_path = None;
    for path in config_paths {
        if path.exists() && path.is_file() {
            found_path = Some(path.clone());
            break;
        }
    }

    // 3. Merge config file if found
    if let Some(config_file) = found_path.and_then(|path| {
        Figment::new()
            .merge(Yaml::file(path))
            .extract::<ConfigFile>()
            .ok()
    }) {
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

    config
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
        let cfg = load_config_with(&["cook-docs"], &[cfg_file.clone()], empty_env).unwrap();
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
                CliAction::Run(_) => panic!("expected Completions action for {name}"),
            }
        }

        assert!(load_action_with(&["cook-docs", "completions"], &[], |_| None).is_err());
        assert!(load_action_with(&["cook-docs", "completions", "tcsh"], &[], |_| None).is_err());
        assert!(load_config_with(&["cook-docs", "completions", "bash"], &[], |_| None).is_err());
    }
}

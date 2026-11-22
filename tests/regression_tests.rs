use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new(prefix: &str) -> Self {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "cook_docs_reg_{}_{}_{}",
            prefix,
            std::process::id(),
            id
        ));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn bin_if_runnable() -> Option<&'static str> {
    let bin = env!("CARGO_BIN_EXE_cook-docs");
    match Command::new(bin).arg("--version").output() {
        Ok(o) if o.status.success() => Some(bin),
        _ => None,
    }
}

fn run_cli(bin: &str, cwd: Option<&Path>, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(bin);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    cmd.args(args);
    for (k, _) in envs {
        cmd.env_remove(k);
    }
    // Isolate from ambient COOK_DOCS_* environment variables
    for (k, _) in std::env::vars() {
        if k.starts_with("COOK_DOCS_") {
            cmd.env_remove(k);
        }
    }
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().unwrap()
}

#[test]
fn regression_cli_help_and_version_contract() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    for flag in ["-v", "--version"] {
        let out = run_cli(bin, None, &[flag], &[]);
        assert!(out.status.success(), "{flag} failed");
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert_eq!(
            stdout.trim(),
            format!("cook-docs {}", env!("PROJECT_VERSION"))
        );
    }

    let expected_usage = if cfg!(windows) {
        "Usage: cook-docs.exe [OPTIONS]"
    } else {
        "Usage: cook-docs [OPTIONS]"
    };

    for flag in ["-h", "--help"] {
        let out = run_cli(bin, None, &[flag], &[]);
        assert!(out.status.success(), "{flag} failed");
        let stdout = String::from_utf8_lossy(&out.stdout);
        for expected in [
            expected_usage,
            "-v, --version",
            "-C, --config <config>",
            "-d, --dry-run",
            "-j, --jsonify",
            "-i, --ignore-file <ignore-file>",
            "-c, --recipe-search-root <recipe-search-root>",
            "-l, --log-level <log-level>",
            "-t, --template-files <template-files>",
            "-w, --word-wrap <word-wrap>",
            "-o, --output-dir <output-dir>",
            "--assets-dir <assets-dir>",
            "--image-ext <image-ext>",
            "--watch",
            "--debounce <MS>",
            "-h, --help",
        ] {
            assert!(
                stdout.contains(expected),
                "help output missing {expected:?}:\n{stdout}"
            );
        }
    }

    let bad = run_cli(bin, None, &["--does-not-exist"], &[]);
    assert!(!bad.status.success());
}

#[test]
fn regression_testdata_golden_markdown_output() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let out = run_cli(
        bin,
        None,
        &["--recipe-search-root", "testdata", "--dry-run"],
        &[],
    );
    assert!(out.status.success());

    let expected = "\
# My Test Recipe

![My Test Recipe](../assets/images/my-test-recipe.png){ loading=lazy }

| :fork_and_knife_with_plate: Serves | :timer_clock: Total Time |
|:----------------------------------:|:-----------------------: |
| 6 | 2.04 days |

## :salt: Ingredients

- 820 g tipo zero flour
- 533 ml water
- 24.6 g salt
- 1.6 g fresh yeast
- 3 cans chopped tomato
- 3 cloves garlic
- 3 tbsp dried oregano
- 18 leaves fresh basil
- 3 packs parma ham
- 3 packs mozzarella

## :cooking: Cookware

- 1 fridge
- 1 oven
- 1 pizza stone
- 1 pan
- 1 spoon
- 1 oven

## :pencil: Instructions

### Step 1

Make 6 pizza balls using tipo zero flour, water, salt and fresh yeast. Put in a fridge for 2 days.

### Step 2

Set oven to max temperature and heat pizza stone for about 40 minutes.

### Step 3

Make some tomato sauce with chopped tomato and garlic and dried oregano. Put on a pan and leave for 15 minutes
occasionally stirring.

### Step 4

Make pizzas putting some tomato sauce with spoon on top of flattened dough. Add fresh basil, parma ham and mozzarella.

### Step 5

Put in an oven for 4 minutes.

## :link: Source

- <https://www.somewebsite.com/pizza-balls>

";

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout, expected);
    assert!(!Path::new("testdata/my-test-recipe.md").exists());
}

#[test]
fn regression_jsonify_schema_and_no_disk_write() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("jsonify");
    let recipe = dir.path().join("My Test Recipe.cook");
    fs::write(
        &recipe,
        fs::read_to_string("testdata/My Test Recipe.cook").unwrap(),
    )
    .unwrap();
    fs::write(dir.path().join("My Test Recipe.png"), b"png").unwrap();

    let out = run_cli(
        bin,
        Some(dir.path()),
        &["--jsonify", "--log-level", "info"],
        &[],
    );
    assert!(out.status.success());
    assert!(
        !dir.path().join("my-test-recipe.md").exists(),
        "--jsonify must not write markdown file"
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: Value = serde_json::from_str(stdout.trim()).expect("valid JSON on stdout");

    assert_eq!(parsed["Info"]["RecipeName"], "My Test Recipe");
    assert_eq!(parsed["Info"]["ImageFileName"], "My Test Recipe.png");
    assert_eq!(parsed["Info"]["ImageSlug"], "my-test-recipe.png");
    assert_eq!(parsed["Metadata"]["servings"], "6");
    assert_eq!(
        parsed["Metadata"]["source"],
        "https://www.somewebsite.com/pizza-balls"
    );
    assert_eq!(parsed["Config"]["Jsonify"], true);
    assert_eq!(parsed["Config"]["DryRun"], false);
    assert_eq!(parsed["Config"]["WordWrap"], 120);
    assert_eq!(parsed["Config"]["IgnoreFile"], ".cookdocsignore");
    assert_eq!(parsed["Config"]["RecipeSearchRoot"], ".");
    assert_eq!(parsed["Config"]["LogLevel"], "info");
    assert_eq!(parsed["Config"]["TemplateFiles"][0], "recipe.md.gotmpl");
    assert_eq!(parsed["Config"]["OutputDir"], "");

    let steps = parsed["Steps"].as_array().unwrap();
    assert_eq!(steps.len(), 5);
    assert!(steps[0]["Directions"].is_string());
    assert!(steps[0]["Timers"].is_array());
    assert!(steps[0]["Cookware"].is_array());
    assert!(steps[0]["Comments"].is_array());
    let first_ing = &steps[0]["Ingredients"][0];
    assert_eq!(first_ing["Name"], "tipo zero flour");
    assert_eq!(first_ing["Amount"]["IsNumeric"], true);
    assert_eq!(first_ing["Amount"]["Quantity"], 820.0);
    assert_eq!(first_ing["Amount"]["QuantityRaw"], "820");
    assert_eq!(first_ing["Amount"]["Unit"], "g");
}

#[test]
fn regression_write_markdown_default_and_output_dir() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("write_out");
    let sub = dir.path().join("desserts");
    fs::create_dir_all(&sub).unwrap();
    let cook = sub.join("Apple Pie.cook");
    fs::write(
        &cook,
        ">> serves: 8\nBake @apples{4} in #dish for ~{30%minutes}.",
    )
    .unwrap();

    // 1. Default output writes next to .cook file with kebab-case name
    let out1 = run_cli(bin, Some(dir.path()), &[], &[]);
    assert!(out1.status.success());
    let in_place_md = sub.join("apple-pie.md");
    assert!(in_place_md.exists(), "expected {}", in_place_md.display());
    let content1 = fs::read_to_string(&in_place_md).unwrap();
    assert!(content1.contains("# Apple Pie"));
    assert!(content1.contains("- 4 apples"));
    assert!(content1.contains("- 1 dish"));
    assert!(content1.contains("| 8 | 30 minutes |"));
    fs::remove_file(&in_place_md).unwrap();

    // 2. --output-dir mirrors relative subdirectories without writing in source dir
    let out_root = dir.path().join("rendered/docs");
    let out2 = run_cli(
        bin,
        Some(dir.path()),
        &["-o", out_root.to_str().unwrap()],
        &[],
    );
    assert!(out2.status.success());
    let mirrored_md = out_root.join("desserts/apple-pie.md");
    assert!(mirrored_md.exists(), "expected {}", mirrored_md.display());
    assert!(!in_place_md.exists());
}

#[test]
fn regression_config_file_env_cli_precedence_and_aliases() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("precedence");
    let recipes_dir = dir.path().join("recipes");
    fs::create_dir_all(&recipes_dir).unwrap();
    fs::write(
        recipes_dir.join("Toast.cook"),
        "Toast @bread{2%slices} until golden brown and crispy on both sides.",
    )
    .unwrap();

    fs::write(
        dir.path().join(".cookdocs.yaml"),
        "\
dryRun: true
recipe-search-root: recipes
word_wrap: 20
logLevel: warn
",
    )
    .unwrap();

    // 1. Config file alone: dryRun=true (prints to stdout, nothing written), wordWrap=20
    let out_cfg = run_cli(bin, Some(dir.path()), &[], &[]);
    assert!(out_cfg.status.success());
    let stdout_cfg = String::from_utf8_lossy(&out_cfg.stdout);
    assert!(stdout_cfg.contains("# Toast"));
    assert!(stdout_cfg.contains("Toast bread until\ngolden brown and"));
    assert!(!recipes_dir.join("toast.md").exists());

    // 2. Env overrides config file: COOK_DOCS_DRY_RUN=false writes file, COOK_DOCS_WORD_WRAP=120 unwraps
    let out_env = run_cli(
        bin,
        Some(dir.path()),
        &[],
        &[
            ("COOK_DOCS_DRY_RUN", "false"),
            ("COOK_DOCS_WORD_WRAP", "120"),
        ],
    );
    assert!(out_env.status.success());
    let written = fs::read_to_string(recipes_dir.join("toast.md")).unwrap();
    assert!(written.contains("Toast bread until golden brown and crispy on both sides."));
    fs::remove_file(recipes_dir.join("toast.md")).unwrap();

    // 3. CLI overrides env and config file
    let out_cli = run_cli(
        bin,
        Some(dir.path()),
        &["-d", "-w", "15"],
        &[
            ("COOK_DOCS_DRY_RUN", "false"),
            ("COOK_DOCS_WORD_WRAP", "120"),
        ],
    );
    assert!(out_cli.status.success());
    let stdout_cli = String::from_utf8_lossy(&out_cli.stdout);
    assert!(stdout_cli.contains("Toast bread\nuntil golden"));
    assert!(!recipes_dir.join("toast.md").exists());

    // 4. .cookdocs.yml works when .cookdocs.yaml is removed
    fs::remove_file(dir.path().join(".cookdocs.yaml")).unwrap();
    fs::write(
        dir.path().join(".cookdocs.yml"),
        "dryRun: true\nrecipeSearchRoot: recipes\n",
    )
    .unwrap();
    let out_yml = run_cli(bin, Some(dir.path()), &[], &[]);
    assert!(out_yml.status.success());
    assert!(String::from_utf8_lossy(&out_yml.stdout).contains("# Toast"));
}

#[test]
fn regression_ignore_file_excludes_recipes_and_directories() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("ignore");
    // find_recipe_file_paths anchors ignore_file to git root when inside a git repo
    let git_init = Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(git_init.status.success());

    fs::create_dir_all(dir.path().join("kept")).unwrap();
    fs::create_dir_all(dir.path().join("ignored_dir")).unwrap();
    fs::write(dir.path().join("kept/Keep Me.cook"), "Boil @water{1%l}.").unwrap();
    fs::write(dir.path().join("kept/Skip Me.cook"), "Fry @egg{1}.").unwrap();
    fs::write(
        dir.path().join("ignored_dir/Hidden.cook"),
        "Steam @rice{1%cup}.",
    )
    .unwrap();
    fs::write(
        dir.path().join(".cookdocsignore"),
        "ignored_dir/\nSkip Me.cook\n",
    )
    .unwrap();

    let out = run_cli(bin, Some(dir.path()), &["--dry-run"], &[]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("# Keep Me"));
    assert!(!stdout.contains("# Skip Me"));
    assert!(!stdout.contains("# Hidden"));

    // Custom -i ignore file overrides .cookdocsignore
    fs::write(dir.path().join(".altignore"), "kept/Keep Me.cook\n").unwrap();
    let out_alt = run_cli(
        bin,
        Some(dir.path()),
        &["--dry-run", "-i", ".altignore"],
        &[],
    );
    assert!(out_alt.status.success());
    let stdout_alt = String::from_utf8_lossy(&out_alt.stdout);
    assert!(!stdout_alt.contains("# Keep Me"));
    assert!(stdout_alt.contains("# Skip Me"));
    assert!(stdout_alt.contains("# Hidden"));
}

#[test]
fn regression_template_composition_fallback_and_variants() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("templates");
    fs::create_dir_all(dir.path().join("shared")).unwrap();
    fs::create_dir_all(dir.path().join("dishes")).unwrap();

    // Shared template overrides cook.headerSection, local template uses it
    fs::write(
        dir.path().join("shared/header.gotmpl"),
        r#"{{ define "cook.headerSection" }}# Recipe: {{ .Info.RecipeName }}{{ end }}"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("dishes/recipe.md.gotmpl"),
        "{{ template \"cook.headerSection\" . }}\n\n{{ template \"cook.tableSection\" . }}\n\n{{ template \"cook.sourceSection\" . }}\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("dishes/Plain Soup.cook"),
        ">> source: Grandma's Book\nSimmer @water{1%l} for ~{10%minutes}.",
    )
    .unwrap();
    // Both .png and .jpg exist -> .jpg takes priority
    fs::write(dir.path().join("dishes/Plain Soup.png"), b"png").unwrap();
    fs::write(dir.path().join("dishes/Plain Soup.jpg"), b"jpg").unwrap();

    let out = run_cli(
        bin,
        Some(dir.path()),
        &["-d", "-t", "./shared/header.gotmpl,recipe.md.gotmpl"],
        &[],
    );
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("# Recipe: Plain Soup"));
    // Without servings/serves metadata, only Total Time column renders
    assert!(
        stdout
            .contains("| :timer_clock: Total Time |\n|:-----------------------: |\n| 10 minutes |")
    );
    assert!(!stdout.contains("Serves"));
    // Non-URL source is not wrapped in <>
    assert!(stdout.contains("- Grandma's Book"));
    assert!(!stdout.contains("<Grandma's Book>"));

    // Missing template falls back to default template, which uses .jpg over .png
    let out_fallback = run_cli(
        bin,
        Some(dir.path()),
        &["-d", "-t", "nonexistent.gotmpl"],
        &[],
    );
    assert!(out_fallback.status.success());
    let stdout_fb = String::from_utf8_lossy(&out_fallback.stdout);
    assert!(stdout_fb.contains("# Plain Soup"));
    assert!(stdout_fb.contains("plain-soup.jpg"));
    assert!(!stdout_fb.contains("plain-soup.png"));
}

#[test]
fn regression_log_level_and_error_resilience() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let out_warn = run_cli(
        bin,
        None,
        &[
            "--recipe-search-root",
            "testdata",
            "--dry-run",
            "--log-level",
            "warn",
        ],
        &[],
    );
    assert!(out_warn.status.success());
    assert!(String::from_utf8_lossy(&out_warn.stdout).contains("# My Test Recipe"));
    assert!(!String::from_utf8_lossy(&out_warn.stderr).contains("INFO"));

    let out_debug = run_cli(
        bin,
        None,
        &[
            "--recipe-search-root",
            "testdata",
            "--dry-run",
            "--log-level",
            "debug",
        ],
        &[],
    );
    assert!(out_debug.status.success());
    assert!(String::from_utf8_lossy(&out_debug.stdout).contains("# My Test Recipe"));
    assert!(String::from_utf8_lossy(&out_debug.stderr).contains("DEBUG"));

    let out_missing_root = run_cli(
        bin,
        None,
        &[
            "--recipe-search-root",
            "/nonexistent/cook-docs-root",
            "--dry-run",
        ],
        &[],
    );
    assert!(out_missing_root.status.success());
}

#[test]
fn regression_completions_subcommand_all_shells() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("completions");
    fs::write(dir.path().join("Sample.cook"), "Boil @water{1%l}.").unwrap();

    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let out = run_cli(bin, Some(dir.path()), &["completions", shell], &[]);
        assert!(
            out.status.success(),
            "completions {shell} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("cook-docs") && stdout.contains("dry-run"),
            "unexpected completions output for {shell}:\n{stdout}"
        );
        assert!(
            !String::from_utf8_lossy(&out.stderr).contains("Found recipes"),
            "completions {shell} must not trigger recipe scanning"
        );
    }

    assert!(!dir.path().join("sample.md").exists());

    let missing = run_cli(bin, Some(dir.path()), &["completions"], &[]);
    assert!(!missing.status.success());

    let invalid = run_cli(bin, Some(dir.path()), &["completions", "tcsh"], &[]);
    assert!(!invalid.status.success());
}

#[test]
fn regression_man_subcommand() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("man");
    fs::write(dir.path().join("Sample.cook"), "Boil @water{1%l}.").unwrap();

    let out = run_cli(bin, Some(dir.path()), &["man"], &[]);
    assert!(
        out.status.success(),
        "man failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(".TH cook-docs 1") && stdout.contains("OPTIONS"),
        "unexpected man output:\n{stdout}"
    );
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("Found recipes"),
        "man must not trigger recipe scanning"
    );
    assert!(!dir.path().join("sample.md").exists());

    let extra = run_cli(bin, Some(dir.path()), &["man", "unexpected"], &[]);
    assert!(!extra.status.success());
}

#[test]
fn regression_config_flag_toml_json_and_env() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("config_formats");
    let recipes_dir = dir.path().join("recipes");
    fs::create_dir_all(&recipes_dir).unwrap();
    fs::write(
        recipes_dir.join("Soup.cook"),
        "Simmer @water{1%l} until hot and steaming.",
    )
    .unwrap();

    let toml_cfg = dir.path().join("custom.toml");
    fs::write(
        &toml_cfg,
        "dry_run = true\nrecipe_search_root = \"recipes\"\nword_wrap = 20\n",
    )
    .unwrap();

    let out_toml = run_cli(
        bin,
        Some(dir.path()),
        &["-C", toml_cfg.to_str().unwrap()],
        &[],
    );
    assert!(out_toml.status.success());
    let stdout_toml = String::from_utf8_lossy(&out_toml.stdout);
    assert!(stdout_toml.contains("# Soup"));
    assert!(stdout_toml.contains("Simmer water until\nhot and steaming."));
    assert!(!recipes_dir.join("soup.md").exists());

    let json_cfg = dir.path().join("custom.json");
    fs::write(
        &json_cfg,
        r#"{"dryRun": true, "recipeSearchRoot": "recipes", "wordWrap": 120}"#,
    )
    .unwrap();

    let out_env_json = run_cli(
        bin,
        Some(dir.path()),
        &[],
        &[("COOK_DOCS_CONFIG", json_cfg.to_str().unwrap())],
    );
    assert!(out_env_json.status.success());
    let stdout_json = String::from_utf8_lossy(&out_env_json.stdout);
    assert!(stdout_json.contains("Simmer water until hot and steaming."));

    let missing_cfg = dir.path().join("missing.toml");
    let out_missing = run_cli(
        bin,
        Some(dir.path()),
        &["--config", missing_cfg.to_str().unwrap()],
        &[],
    );
    assert!(!out_missing.status.success());
}

#[test]
fn regression_doctor_subcommand_text_and_json() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("doctor");
    let recipes_dir = dir.path().join("recipes");
    let out_dir = dir.path().join("out");
    fs::create_dir_all(&recipes_dir).unwrap();
    fs::create_dir_all(&out_dir).unwrap();
    fs::write(recipes_dir.join("Toast.cook"), "Toast @bread{2%slices}.").unwrap();
    fs::write(dir.path().join(".cookdocsignore"), "ignored/\n").unwrap();

    let cfg_path = dir.path().join("doctor.toml");
    fs::write(
        &cfg_path,
        format!(
            "recipe_search_root = {:?}\noutput_dir = {:?}\n",
            recipes_dir, out_dir
        ),
    )
    .unwrap();

    // 1. Healthy text report via `doctor`
    let out_ok = run_cli(
        bin,
        Some(dir.path()),
        &["doctor", "-C", cfg_path.to_str().unwrap()],
        &[],
    );
    assert!(
        out_ok.status.success(),
        "doctor failed: {}",
        String::from_utf8_lossy(&out_ok.stderr)
    );
    let stdout_ok = String::from_utf8_lossy(&out_ok.stdout);
    assert!(stdout_ok.contains("[OK]"));
    assert!(stdout_ok.contains("Doctor summary:"));
    assert!(stdout_ok.contains("0 error(s)."));

    // 2. Healthy JSON report via `doc` alias
    let out_json = run_cli(
        bin,
        Some(dir.path()),
        &["doc", "--json", "-C", cfg_path.to_str().unwrap()],
        &[],
    );
    assert!(out_json.status.success());
    let report: Value =
        serde_json::from_slice(&out_json.stdout).expect("doctor --json must output valid JSON");
    assert_eq!(report["healthy"], true);
    assert_eq!(report["errors"], 0);
    let checks = report["checks"].as_array().unwrap();
    assert!(checks.iter().any(|c| c["category"] == "config"));
    assert!(checks.iter().any(|c| c["category"] == "recipe_search_root"));
    assert!(checks.iter().any(|c| c["category"] == "ignore_file"));
    assert!(checks.iter().any(|c| c["category"] == "templates"));
    assert!(checks.iter().any(|c| c["category"] == "output_dir"));

    // 3. Unhealthy: broken template compilation & missing search root -> exit non-zero
    fs::write(recipes_dir.join("recipe.md.gotmpl"), "{{ .Unclosed").unwrap();
    let out_bad_tmpl = run_cli(
        bin,
        Some(dir.path()),
        &["doctor", "--json", "-C", cfg_path.to_str().unwrap()],
        &[],
    );
    assert!(!out_bad_tmpl.status.success());
    let bad_report: Value = serde_json::from_slice(&out_bad_tmpl.stdout).unwrap();
    assert_eq!(bad_report["healthy"], false);
    assert!(bad_report["errors"].as_u64().unwrap() >= 1);

    let out_missing_cfg = run_cli(
        bin,
        Some(dir.path()),
        &["doctor", "--json", "-C", "/nonexistent/cookdocs.toml"],
        &[],
    );
    assert!(!out_missing_cfg.status.success());
    let missing_report: Value = serde_json::from_slice(&out_missing_cfg.stdout).unwrap();
    assert_eq!(missing_report["healthy"], false);
    assert_eq!(missing_report["checks"][0]["category"], "config");
}

#[test]
fn regression_self_update_help_and_non_interactive_guard() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    for sub in ["self-update", "update"] {
        let out = run_cli(bin, None, &[sub, "--help"], &[]);
        assert!(out.status.success(), "{sub} --help failed");
        let stdout = String::from_utf8_lossy(&out.stdout);
        for expected in ["--check", "-y, --yes", "--version <VERSION>"] {
            assert!(
                stdout.contains(expected),
                "{sub} help missing {expected:?}:\n{stdout}"
            );
        }
    }

    let dir = TempDir::new("self_update_guard");
    fs::write(dir.path().join("Sample.cook"), "Boil @water{1%l}.").unwrap();
    let out = run_cli(bin, Some(dir.path()), &["self-update"], &[]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--yes"),
        "expected non-interactive --yes error, got:\n{stderr}"
    );
    assert!(
        !dir.path().join("sample.md").exists(),
        "self-update must not trigger recipe rendering"
    );
}

#[test]
fn regression_watch_help_and_missing_root() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let out = run_cli(bin, None, &["watch", "--help"], &[]);
    assert!(out.status.success(), "watch --help failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    for expected in ["--debounce <MS>", "--dry-run", "--recipe-search-root"] {
        assert!(
            stdout.contains(expected),
            "watch help missing {expected:?}:\n{stdout}"
        );
    }

    let missing = run_cli(
        bin,
        None,
        &["watch", "-c", "nonexistent_watch_dir_99999"],
        &[],
    );
    assert!(!missing.status.success());
    let stderr = String::from_utf8_lossy(&missing.stderr);
    assert!(
        stderr.contains("does not exist"),
        "expected missing search root error, got:\n{stderr}"
    );
}

#[test]
fn regression_jsonify_multi_recipe_stdout_array_and_empty() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("jsonify_multi");
    fs::write(dir.path().join("Alpha.cook"), "Boil @water{1%l}.").unwrap();
    fs::write(dir.path().join("Beta.cook"), "Season with @salt{}.").unwrap();

    let out = run_cli(bin, Some(dir.path()), &["--jsonify"], &[]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: Value = serde_json::from_str(stdout.trim()).expect("valid JSON array on stdout");
    let arr = parsed
        .as_array()
        .expect("expected JSON array for multiple recipes");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["Info"]["RecipeName"], "Alpha");
    assert_eq!(arr[1]["Info"]["RecipeName"], "Beta");

    // Empty dir returns valid empty JSON array
    let empty_dir = TempDir::new("jsonify_empty");
    let out_empty = run_cli(bin, Some(empty_dir.path()), &["--jsonify"], &[]);
    assert!(out_empty.status.success());
    let empty_stdout = String::from_utf8_lossy(&out_empty.stdout);
    let empty_parsed: Value =
        serde_json::from_str(empty_stdout.trim()).expect("valid JSON array for 0 recipes");
    assert_eq!(empty_parsed.as_array().unwrap().len(), 0);
}

#[test]
fn regression_bare_ingredient_quantity_zero_renders_some() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("bare_ing");
    fs::write(
        dir.path().join("Seasoning.cook"),
        "Season with @salt{} and @black pepper in a #bowl{}.",
    )
    .unwrap();

    let out_json = run_cli(bin, Some(dir.path()), &["--jsonify"], &[]);
    assert!(out_json.status.success());
    let parsed: Value =
        serde_json::from_str(String::from_utf8_lossy(&out_json.stdout).trim()).unwrap();
    let ings = parsed["Steps"][0]["Ingredients"].as_array().unwrap();
    assert_eq!(ings[0]["Name"], "salt");
    assert_eq!(ings[0]["Amount"]["IsNumeric"], false);
    assert_eq!(ings[0]["Amount"]["Quantity"], 0.0);
    assert_eq!(ings[1]["Name"], "black");
    assert_eq!(ings[1]["Amount"]["Quantity"], 0.0);
    assert_eq!(parsed["Steps"][0]["Cookware"][0]["Quantity"], 1.0);

    let out_md = run_cli(bin, Some(dir.path()), &["--dry-run"], &[]);
    assert!(out_md.status.success());
    let md = String::from_utf8_lossy(&out_md.stdout);
    assert!(
        md.contains("- some salt"),
        "expected '- some salt' in:\n{md}"
    );
    assert!(!md.contains("- 1 salt"), "unexpected '- 1 salt' in:\n{md}");
    assert!(md.contains("- 1 bowl"), "expected '- 1 bowl' in:\n{md}");
}

#[test]
fn regression_positional_recipe_files_with_output_dir() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("positional_files");
    fs::create_dir_all(dir.path().join("cook/breads")).unwrap();
    fs::create_dir_all(dir.path().join("cook/soups")).unwrap();
    fs::write(
        dir.path().join("cook/breads/My Bagels.cook"),
        "Bake @flour{500%g} for ~{20%minutes}.",
    )
    .unwrap();
    fs::write(
        dir.path().join("cook/soups/Chili.cook"),
        "Simmer @beans{2%cans} for ~{30%minutes}.",
    )
    .unwrap();

    // 1. Positional file relative to cwd with -c cook -o docs
    let out = run_cli(
        bin,
        Some(dir.path()),
        &["-c", "cook", "-o", "docs", "cook/breads/My Bagels.cook"],
        &[],
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(dir.path().join("docs/breads/my-bagels.md").exists());
    assert!(!dir.path().join("docs/soups/chili.md").exists());
    assert!(!dir.path().join("cook/breads/my-bagels.md").exists());

    // 2. Positional file relative to search root (-c cook)
    let out_rel = run_cli(
        bin,
        Some(dir.path()),
        &["-c", "cook", "-o", "docs2", "soups/Chili.cook"],
        &[],
    );
    assert!(
        out_rel.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out_rel.stderr)
    );
    assert!(dir.path().join("docs2/soups/chili.md").exists());
    assert!(!dir.path().join("docs2/breads/my-bagels.md").exists());

    // 3. Nonexistent positional file fails cleanly
    let bad = run_cli(
        bin,
        Some(dir.path()),
        &["-c", "cook", "cook/breads/Nope.cook"],
        &[],
    );
    assert!(!bad.status.success());
}

#[test]
fn regression_webp_jpeg_image_slug_and_assets_dir() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("image_assets");
    fs::create_dir_all(dir.path().join("cook/breads")).unwrap();
    fs::write(
        dir.path().join("cook/breads/Sourdough Loaf.cook"),
        "Bake @flour{500%g} for ~{35%minutes}.",
    )
    .unwrap();
    fs::write(
        dir.path().join("cook/breads/Sourdough Loaf.jpeg"),
        b"jpeg-bytes",
    )
    .unwrap();
    fs::write(
        dir.path().join("cook/breads/recipe.md.gotmpl"),
        "slug={{ .Info.ImageSlug }}\nname={{ .Info.ImageFileName }}\n",
    )
    .unwrap();

    // 1. .jpeg detected, --image-ext overrides ImageSlug only, --assets-dir copies with source ext
    let out = run_cli(
        bin,
        Some(dir.path()),
        &[
            "-c",
            "cook",
            "-o",
            "docs",
            "--assets-dir",
            "docs/assets/images",
            "--image-ext",
            "webp",
        ],
        &[],
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let md = fs::read_to_string(dir.path().join("docs/breads/sourdough-loaf.md")).unwrap();
    assert!(md.contains("slug=sourdough-loaf.webp"), "md:\n{md}");
    assert!(md.contains("name=Sourdough Loaf.jpeg"), "md:\n{md}");
    let copied = dir.path().join("docs/assets/images/sourdough-loaf.jpeg");
    assert!(
        copied.exists(),
        "expected copied asset at {}",
        copied.display()
    );
    assert_eq!(fs::read(&copied).unwrap(), b"jpeg-bytes");
    assert!(
        !dir.path()
            .join("docs/assets/images/sourdough-loaf.webp")
            .exists()
    );

    // 2. .webp takes priority over .jpeg and .jpg; --dry-run does not copy assets
    fs::write(
        dir.path().join("cook/breads/Sourdough Loaf.jpg"),
        b"jpg-bytes",
    )
    .unwrap();
    fs::write(
        dir.path().join("cook/breads/Sourdough Loaf.webp"),
        b"webp-bytes",
    )
    .unwrap();
    let out_dry = run_cli(
        bin,
        Some(dir.path()),
        &["-c", "cook", "-d", "--assets-dir", "dry_assets"],
        &[],
    );
    assert!(out_dry.status.success());
    let dry_stdout = String::from_utf8_lossy(&out_dry.stdout);
    assert!(
        dry_stdout.contains("slug=sourdough-loaf.webp"),
        "dry:\n{dry_stdout}"
    );
    assert!(
        dry_stdout.contains("name=Sourdough Loaf.webp"),
        "dry:\n{dry_stdout}"
    );
    assert!(!dir.path().join("dry_assets").exists());
}

#[test]
fn regression_structured_frontmatter_info_fields_and_defaults() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("frontmatter_info");
    fs::create_dir_all(dir.path().join("cook/desserts/cake")).unwrap();
    fs::write(
        dir.path().join("cook/desserts/cake/Lemon Tart.cook"),
        "\
---
title: \":pie: Lemon Tart\"
serves: 8
total time: 1 hour 15 minutes
comments: true
tags:
  - dessert
  - citrus
---
Bake @lemon{2} for ~{20%minutes}.
",
    )
    .unwrap();
    fs::write(dir.path().join("cook/desserts/cake/Lemon Tart.webp"), b"w").unwrap();
    fs::write(
        dir.path().join("cook/Simple Toast.cook"),
        "Toast @bread{2%slices} for ~{3%minutes}.",
    )
    .unwrap();

    // 1. JSON output checks structured Frontmatter + new Info fields
    let out_json = run_cli(bin, Some(dir.path()), &["-c", "cook", "--jsonify"], &[]);
    assert!(out_json.status.success());
    let arr: Value =
        serde_json::from_str(String::from_utf8_lossy(&out_json.stdout).trim()).unwrap();
    let recipes = arr.as_array().unwrap();
    assert_eq!(recipes.len(), 2);

    let tart = recipes
        .iter()
        .find(|r| r["Info"]["RecipeName"] == "Lemon Tart")
        .unwrap();
    assert_eq!(tart["Info"]["RecipeSlug"], "lemon-tart");
    assert_eq!(tart["Info"]["RelRoot"], "../..");
    assert_eq!(
        tart["Info"]["Categories"],
        serde_json::json!(["desserts", "cake"])
    );
    assert_eq!(
        tart["Frontmatter"]["tags"],
        serde_json::json!(["dessert", "citrus"])
    );
    assert_eq!(tart["Frontmatter"]["serves"], 8);
    assert_eq!(tart["Frontmatter"]["comments"], true);
    assert_eq!(tart["Metadata"]["serves"], "8");

    let toast = recipes
        .iter()
        .find(|r| r["Info"]["RecipeName"] == "Simple Toast")
        .unwrap();
    assert_eq!(toast["Info"]["RecipeSlug"], "simple-toast");
    assert_eq!(toast["Info"]["RelRoot"], ".");
    assert_eq!(toast["Info"]["Categories"], serde_json::json!([]));

    // 2. Default template uses Metadata.title and Metadata["total time"]
    let out_md = run_cli(
        bin,
        Some(dir.path()),
        &["-c", "cook", "-d", "cook/desserts/cake/Lemon Tart.cook"],
        &[],
    );
    assert!(out_md.status.success());
    let md = String::from_utf8_lossy(&out_md.stdout);
    assert!(md.contains("# :pie: Lemon Tart"), "md:\n{md}");
    assert!(md.contains("| 8 | 1 hour 15 minutes |"), "md:\n{md}");
}

#[test]
fn regression_parent_template_fallback_and_to_yaml() {
    let Some(bin) = bin_if_runnable() else {
        return;
    };

    let dir = TempDir::new("parent_template");
    fs::create_dir_all(dir.path().join("cook/desserts/cake")).unwrap();
    fs::create_dir_all(dir.path().join("cook/breads")).unwrap();

    // Template outside search root must NOT be picked up
    fs::write(dir.path().join("recipe.md.gotmpl"), "OUTSIDE_ROOT_LEAK\n").unwrap();

    // Root template inside search root
    fs::write(
        dir.path().join("cook/recipe.md.gotmpl"),
        "\
---
categories:
{{ toYaml .Info.Categories }}
tags:
{{ toYaml .Frontmatter.tags }}
---
{{ template \"cook.headerSection\" . }}
img={{ .Info.RelRoot }}/assets/images/{{ .Info.ImageSlug }}
",
    )
    .unwrap();

    // Local override in cook/breads/
    fs::write(
        dir.path().join("cook/breads/recipe.md.gotmpl"),
        "LOCAL_BREAD={{ .Info.RecipeSlug }}|{{ .Info.RelRoot }}\n",
    )
    .unwrap();

    fs::write(
        dir.path().join("cook/desserts/cake/Lemon Tart.cook"),
        "\
---
title: \":pie: Lemon Tart\"
tags:
  - dessert
  - citrus
---
Bake @lemon{2}.
",
    )
    .unwrap();
    fs::write(dir.path().join("cook/desserts/cake/Lemon Tart.webp"), b"w").unwrap();
    fs::write(
        dir.path().join("cook/breads/Bagels.cook"),
        "Bake @flour{500%g}.",
    )
    .unwrap();

    // 1. Nested recipe inherits cook/recipe.md.gotmpl
    let out_tart = run_cli(
        bin,
        Some(dir.path()),
        &["-c", "cook", "-d", "cook/desserts/cake/Lemon Tart.cook"],
        &[],
    );
    assert!(out_tart.status.success());
    let tart_md = String::from_utf8_lossy(&out_tart.stdout);
    assert!(tart_md.contains("- desserts\n- cake"), "tart:\n{tart_md}");
    assert!(tart_md.contains("- dessert\n- citrus"), "tart:\n{tart_md}");
    assert!(tart_md.contains("# :pie: Lemon Tart"), "tart:\n{tart_md}");
    assert!(
        tart_md.contains("img=../../assets/images/lemon-tart.webp"),
        "tart:\n{tart_md}"
    );
    assert!(!tart_md.contains("OUTSIDE_ROOT_LEAK"));

    // 2. Local override wins over root template
    let out_bagels = run_cli(
        bin,
        Some(dir.path()),
        &["-c", "cook", "-d", "cook/breads/Bagels.cook"],
        &[],
    );
    assert!(out_bagels.status.success());
    let bagels_md = String::from_utf8_lossy(&out_bagels.stdout);
    assert!(
        bagels_md.contains("LOCAL_BREAD=bagels|.."),
        "bagels:\n{bagels_md}"
    );

    // 3. Removing cook/recipe.md.gotmpl must NOT walk above search root to dir/recipe.md.gotmpl
    fs::remove_file(dir.path().join("cook/recipe.md.gotmpl")).unwrap();
    let out_bounded = run_cli(
        bin,
        Some(dir.path()),
        &["-c", "cook", "-d", "cook/desserts/cake/Lemon Tart.cook"],
        &[],
    );
    assert!(out_bounded.status.success());
    let bounded_md = String::from_utf8_lossy(&out_bounded.stdout);
    assert!(
        !bounded_md.contains("OUTSIDE_ROOT_LEAK"),
        "leaked:\n{bounded_md}"
    );
    assert!(
        bounded_md.contains("# :pie: Lemon Tart"),
        "bounded:\n{bounded_md}"
    );
}

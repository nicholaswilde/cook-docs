use crate::types::Config;
use ignore::gitignore::GitignoreBuilder;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static PROBE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckSeverity {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub category: String,
    pub severity: CheckSeverity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
    pub total_checks: usize,
    pub ok: usize,
    pub warnings: usize,
    pub errors: usize,
    pub healthy: bool,
}

impl DoctorReport {
    /// Build a `DoctorReport` summary from individual checks.
    pub fn new(checks: Vec<DoctorCheck>) -> Self {
        let ok = checks
            .iter()
            .filter(|c| c.severity == CheckSeverity::Ok)
            .count();
        let warnings = checks
            .iter()
            .filter(|c| c.severity == CheckSeverity::Warning)
            .count();
        let errors = checks
            .iter()
            .filter(|c| c.severity == CheckSeverity::Error)
            .count();
        let total_checks = checks.len();
        let healthy = errors == 0;
        Self {
            checks,
            total_checks,
            ok,
            warnings,
            errors,
            healthy,
        }
    }
}

/// Run all diagnostic checks for `cook-docs`.
pub fn run_doctor(
    config_path: Option<&Path>,
    config_result: &Result<Config, String>,
) -> DoctorReport {
    let mut checks = Vec::new();

    match config_result {
        Ok(config) => {
            if let Some(path) = config_path {
                checks.push(DoctorCheck {
                    category: "config".into(),
                    severity: CheckSeverity::Ok,
                    message: format!("Configuration file valid: {}", path.display()),
                    path: Some(path.to_path_buf()),
                });
            } else {
                checks.push(DoctorCheck {
                    category: "config".into(),
                    severity: CheckSeverity::Ok,
                    message: "No custom configuration file found (using defaults)".into(),
                    path: None,
                });
            }

            let search_root = resolve_search_root(&config.recipe_search_root);
            let recipes = check_recipe_search_root(&search_root, &config.ignore_file, &mut checks);
            checks.extend(check_ignore_file(&config.ignore_file));
            checks.extend(check_templates(
                &search_root,
                &recipes,
                &config.template_files,
            ));
            checks.extend(check_output_dir(&config.output_dir, &search_root));
        }
        Err(err) => {
            checks.push(DoctorCheck {
                category: "config".into(),
                severity: CheckSeverity::Error,
                message: err.clone(),
                path: config_path.map(Path::to_path_buf),
            });
        }
    }

    DoctorReport::new(checks)
}

fn resolve_search_root(recipe_search_root: &str) -> PathBuf {
    let root = Path::new(recipe_search_root);
    if root.is_absolute() {
        root.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(root)
    }
}

fn check_recipe_search_root(
    search_root: &Path,
    ignore_file: &str,
    checks: &mut Vec<DoctorCheck>,
) -> Vec<PathBuf> {
    if !search_root.exists() {
        checks.push(DoctorCheck {
            category: "recipe_search_root".into(),
            severity: CheckSeverity::Error,
            message: format!(
                "Recipe search root does not exist: {}",
                search_root.display()
            ),
            path: Some(search_root.to_path_buf()),
        });
        return Vec::new();
    }

    if !search_root.is_dir() {
        checks.push(DoctorCheck {
            category: "recipe_search_root".into(),
            severity: CheckSeverity::Error,
            message: format!(
                "Recipe search root is not a directory: {}",
                search_root.display()
            ),
            path: Some(search_root.to_path_buf()),
        });
        return Vec::new();
    }

    let search_root_str = search_root.to_string_lossy();
    let recipes = crate::cook::find_recipe_file_paths(&search_root_str, ignore_file);
    if recipes.is_empty() {
        checks.push(DoctorCheck {
            category: "recipe_search_root".into(),
            severity: CheckSeverity::Warning,
            message: format!(
                "Recipe search root valid ({}), but 0 .cook files found",
                search_root.display()
            ),
            path: Some(search_root.to_path_buf()),
        });
    } else {
        checks.push(DoctorCheck {
            category: "recipe_search_root".into(),
            severity: CheckSeverity::Ok,
            message: format!(
                "Recipe search root valid ({}): {} recipe(s) found",
                search_root.display(),
                recipes.len()
            ),
            path: Some(search_root.to_path_buf()),
        });
    }
    recipes
}

fn check_ignore_file(ignore_file: &str) -> Vec<DoctorCheck> {
    if ignore_file.trim().is_empty() {
        return vec![DoctorCheck {
            category: "ignore_file".into(),
            severity: CheckSeverity::Ok,
            message: "Ignore file disabled (empty ignore_file setting)".into(),
            path: None,
        }];
    }

    let base_dir = crate::cook::find_git_repository_root().unwrap_or_else(|| {
        Path::new(".")
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from("."))
    });
    check_ignore_file_in(&base_dir, ignore_file)
}

fn check_ignore_file_in(base_dir: &Path, ignore_file: &str) -> Vec<DoctorCheck> {
    let ignore_path = base_dir.join(ignore_file);

    if !ignore_path.exists() {
        let (severity, msg) = if ignore_file == ".cookdocsignore" {
            (
                CheckSeverity::Ok,
                format!(
                    "No ignore file found at {} (optional)",
                    ignore_path.display()
                ),
            )
        } else {
            (
                CheckSeverity::Warning,
                format!(
                    "Configured ignore file not found: {}",
                    ignore_path.display()
                ),
            )
        };
        return vec![DoctorCheck {
            category: "ignore_file".into(),
            severity,
            message: msg,
            path: Some(ignore_path),
        }];
    }

    if !ignore_path.is_file() {
        return vec![DoctorCheck {
            category: "ignore_file".into(),
            severity: CheckSeverity::Error,
            message: format!(
                "Ignore file path is not a regular file: {}",
                ignore_path.display()
            ),
            path: Some(ignore_path),
        }];
    }

    let mut builder = GitignoreBuilder::new(base_dir);
    if let Some(err) = builder.add(&ignore_path) {
        return vec![DoctorCheck {
            category: "ignore_file".into(),
            severity: CheckSeverity::Error,
            message: format!(
                "Failed to parse ignore file '{}': {}",
                ignore_path.display(),
                err
            ),
            path: Some(ignore_path),
        }];
    }
    if let Err(err) = builder.build() {
        return vec![DoctorCheck {
            category: "ignore_file".into(),
            severity: CheckSeverity::Error,
            message: format!(
                "Invalid ignore rules in '{}': {}",
                ignore_path.display(),
                err
            ),
            path: Some(ignore_path),
        }];
    }

    vec![DoctorCheck {
        category: "ignore_file".into(),
        severity: CheckSeverity::Ok,
        message: format!("Ignore file valid: {}", ignore_path.display()),
        path: Some(ignore_path),
    }]
}

fn check_templates(
    search_root: &Path,
    recipes: &[PathBuf],
    template_files: &[String],
) -> Vec<DoctorCheck> {
    let mut checks = Vec::new();
    let search_root_str = search_root.to_string_lossy();

    // Check explicit non-base template paths for missing files
    for template_file in template_files {
        if !crate::cook::is_base_filename(template_file) {
            let resolved = crate::document::resolve_template_path(
                &search_root_str,
                &search_root.join("__probe__.cook").to_string_lossy(),
                template_file,
            );
            if !resolved.exists() || !resolved.is_file() {
                checks.push(DoctorCheck {
                    category: "templates".into(),
                    severity: CheckSeverity::Warning,
                    message: format!(
                        "Configured template file not found (will fall back to built-in default): {}",
                        resolved.display()
                    ),
                    path: Some(resolved),
                });
            }
        }
    }

    // Compile templates for all discovered recipes, or once against search_root when none exist
    let targets: Vec<PathBuf> = if recipes.is_empty() {
        vec![search_root.join("__probe__.cook")]
    } else {
        recipes.to_vec()
    };

    let mut compiled_count = 0usize;
    let mut had_error = false;
    for recipe_path in &targets {
        let recipe_str = recipe_path.to_string_lossy();
        match crate::document::build_documentation_template(
            &search_root_str,
            &recipe_str,
            template_files,
        ) {
            Ok(_) => compiled_count += 1,
            Err(err) => {
                had_error = true;
                checks.push(DoctorCheck {
                    category: "templates".into(),
                    severity: CheckSeverity::Error,
                    message: format!("Template compilation failed for {}: {}", recipe_str, err),
                    path: Some(recipe_path.clone()),
                });
            }
        }
    }

    if !had_error {
        let detail = if recipes.is_empty() {
            "built-in/configured templates valid".to_string()
        } else {
            format!("compiled templates for {} recipe(s)", compiled_count)
        };
        checks.push(DoctorCheck {
            category: "templates".into(),
            severity: CheckSeverity::Ok,
            message: format!("Templates valid ({detail})"),
            path: None,
        });
    }

    checks
}

fn check_dir_writable(dir: &Path) -> Result<(), String> {
    let id = PROBE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let probe = dir.join(format!(
        ".cook_docs_doctor_probe_{}_{}",
        std::process::id(),
        id
    ));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
        .map_err(|e| e.to_string())?;
    drop(file);
    let _ = fs::remove_file(&probe);
    Ok(())
}

fn check_output_dir(output_dir: &str, search_root: &Path) -> Vec<DoctorCheck> {
    if output_dir.trim().is_empty() {
        if !search_root.is_dir() {
            return Vec::new();
        }
        return match check_dir_writable(search_root) {
            Ok(()) => vec![DoctorCheck {
                category: "output_dir".into(),
                severity: CheckSeverity::Ok,
                message: format!(
                    "Output destination (in-place under {}) is writable",
                    search_root.display()
                ),
                path: Some(search_root.to_path_buf()),
            }],
            Err(err) => vec![DoctorCheck {
                category: "output_dir".into(),
                severity: CheckSeverity::Error,
                message: format!(
                    "Output destination (in-place under {}) is not writable: {}",
                    search_root.display(),
                    err
                ),
                path: Some(search_root.to_path_buf()),
            }],
        };
    }

    let out_path = Path::new(output_dir);
    if out_path.exists() {
        if !out_path.is_dir() {
            return vec![DoctorCheck {
                category: "output_dir".into(),
                severity: CheckSeverity::Error,
                message: format!("Output path is not a directory: {}", out_path.display()),
                path: Some(out_path.to_path_buf()),
            }];
        }
        return match check_dir_writable(out_path) {
            Ok(()) => vec![DoctorCheck {
                category: "output_dir".into(),
                severity: CheckSeverity::Ok,
                message: format!("Output directory is writable: {}", out_path.display()),
                path: Some(out_path.to_path_buf()),
            }],
            Err(err) => vec![DoctorCheck {
                category: "output_dir".into(),
                severity: CheckSeverity::Error,
                message: format!(
                    "Output directory is not writable ({}): {}",
                    out_path.display(),
                    err
                ),
                path: Some(out_path.to_path_buf()),
            }],
        };
    }

    // Walk ancestors to find nearest existing directory for writability check
    let mut ancestor = out_path.parent();
    while let Some(p) = ancestor {
        if p.as_os_str().is_empty() {
            let dot = Path::new(".");
            return match check_dir_writable(dot) {
                Ok(()) => vec![DoctorCheck {
                    category: "output_dir".into(),
                    severity: CheckSeverity::Ok,
                    message: format!(
                        "Output directory '{}' does not exist yet (working directory is writable)",
                        out_path.display()
                    ),
                    path: Some(out_path.to_path_buf()),
                }],
                Err(err) => vec![DoctorCheck {
                    category: "output_dir".into(),
                    severity: CheckSeverity::Error,
                    message: format!(
                        "Output directory '{}' cannot be created (working directory not writable: {})",
                        out_path.display(),
                        err
                    ),
                    path: Some(out_path.to_path_buf()),
                }],
            };
        }
        if p.exists() {
            if !p.is_dir() {
                return vec![DoctorCheck {
                    category: "output_dir".into(),
                    severity: CheckSeverity::Error,
                    message: format!(
                        "Output directory ancestor '{}' is not a directory",
                        p.display()
                    ),
                    path: Some(p.to_path_buf()),
                }];
            }
            return match check_dir_writable(p) {
                Ok(()) => vec![DoctorCheck {
                    category: "output_dir".into(),
                    severity: CheckSeverity::Ok,
                    message: format!(
                        "Output directory '{}' does not exist yet (parent '{}' is writable)",
                        out_path.display(),
                        p.display()
                    ),
                    path: Some(out_path.to_path_buf()),
                }],
                Err(err) => vec![DoctorCheck {
                    category: "output_dir".into(),
                    severity: CheckSeverity::Error,
                    message: format!(
                        "Output directory '{}' cannot be created (parent '{}' not writable: {})",
                        out_path.display(),
                        p.display(),
                        err
                    ),
                    path: Some(out_path.to_path_buf()),
                }],
            };
        }
        ancestor = p.parent();
    }

    vec![DoctorCheck {
        category: "output_dir".into(),
        severity: CheckSeverity::Error,
        message: format!(
            "Cannot resolve writable parent for output directory '{}'",
            out_path.display()
        ),
        path: Some(out_path.to_path_buf()),
    }]
}

/// Format a `DoctorReport` into a human-readable report string.
pub fn format_doctor_report(report: &DoctorReport, colorize: bool) -> String {
    let mut out = String::new();

    for check in &report.checks {
        let tag = match check.severity {
            CheckSeverity::Ok => {
                if colorize {
                    "\x1b[32m[OK]\x1b[0m   "
                } else {
                    "[OK]   "
                }
            }
            CheckSeverity::Warning => {
                if colorize {
                    "\x1b[33m[WARN]\x1b[0m "
                } else {
                    "[WARN] "
                }
            }
            CheckSeverity::Error => {
                if colorize {
                    "\x1b[31m[ERROR]\x1b[0m"
                } else {
                    "[ERROR]"
                }
            }
        };
        out.push_str(&format!("{} {}\n", tag, check.message));
    }

    out.push_str(&format!(
        "\nDoctor summary: {} passed, {} warning(s), {} error(s).\n",
        report.ok, report.warnings, report.errors
    ));

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir(prefix: &str) -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "cook_docs_doc_{}_{}_{}",
            prefix,
            std::process::id(),
            id
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_doctor_report_counts_and_healthy() {
        let checks = vec![
            DoctorCheck {
                category: "config".into(),
                severity: CheckSeverity::Ok,
                message: "ok".into(),
                path: None,
            },
            DoctorCheck {
                category: "templates".into(),
                severity: CheckSeverity::Warning,
                message: "warn".into(),
                path: None,
            },
        ];
        let report = DoctorReport::new(checks);
        assert_eq!(report.total_checks, 2);
        assert_eq!(report.ok, 1);
        assert_eq!(report.warnings, 1);
        assert_eq!(report.errors, 0);
        assert!(report.healthy);

        let report_err = DoctorReport::new(vec![DoctorCheck {
            category: "config".into(),
            severity: CheckSeverity::Error,
            message: "err".into(),
            path: None,
        }]);
        assert_eq!(report_err.errors, 1);
        assert!(!report_err.healthy);
    }

    #[test]
    fn test_format_doctor_report_color_and_plain() {
        let report = DoctorReport::new(vec![
            DoctorCheck {
                category: "config".into(),
                severity: CheckSeverity::Ok,
                message: "Config ok".into(),
                path: None,
            },
            DoctorCheck {
                category: "ignore_file".into(),
                severity: CheckSeverity::Warning,
                message: "Ignore warn".into(),
                path: None,
            },
            DoctorCheck {
                category: "templates".into(),
                severity: CheckSeverity::Error,
                message: "Template err".into(),
                path: None,
            },
        ]);

        let plain = format_doctor_report(&report, false);
        assert!(plain.contains("[OK]    Config ok"));
        assert!(plain.contains("[WARN]  Ignore warn"));
        assert!(plain.contains("[ERROR] Template err"));
        assert!(plain.contains("Doctor summary: 1 passed, 1 warning(s), 1 error(s)."));

        let colored = format_doctor_report(&report, true);
        assert!(colored.contains("\x1b[32m[OK]\x1b[0m"));
        assert!(colored.contains("\x1b[33m[WARN]\x1b[0m"));
        assert!(colored.contains("\x1b[31m[ERROR]\x1b[0m"));
    }

    #[test]
    fn test_run_doctor_healthy_and_error_states() {
        let dir = temp_dir("run");
        let recipes = dir.join("recipes");
        let out = dir.join("out/nested");
        fs::create_dir_all(&recipes).unwrap();
        fs::write(recipes.join("Dish.cook"), "Boil @water{1%l}.").unwrap();

        let cfg = Config {
            dry_run: false,
            jsonify: false,
            ignore_file: "".to_string(),
            recipe_search_root: recipes.to_string_lossy().to_string(),
            log_level: "info".to_string(),
            template_files: vec!["recipe.md.gotmpl".to_string()],
            word_wrap: 120,
            output_dir: out.to_string_lossy().to_string(),
        };

        let report_ok = run_doctor(None, &Ok(cfg.clone()));
        assert!(report_ok.healthy);
        assert_eq!(report_ok.errors, 0);

        let bad_cfg = Config {
            recipe_search_root: dir.join("missing_root").to_string_lossy().to_string(),
            ..cfg
        };
        let report_bad = run_doctor(None, &Ok(bad_cfg));
        assert!(!report_bad.healthy);
        assert!(report_bad.errors >= 1);

        let report_cfg_err = run_doctor(
            Some(&dir.join("bad.toml")),
            &Err("Failed to parse config".to_string()),
        );
        assert!(!report_cfg_err.healthy);
        assert_eq!(report_cfg_err.checks.len(), 1);
        assert_eq!(report_cfg_err.checks[0].category, "config");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_search_root_and_recipe_root_edge_cases() {
        let rel = resolve_search_root("testdata");
        assert!(rel.is_absolute());
        assert!(rel.ends_with("testdata"));

        let dir = temp_dir("root_edges");
        let file_root = dir.join("not_a_dir.cook");
        fs::write(&file_root, "").unwrap();

        let mut checks = Vec::new();
        let found = check_recipe_search_root(&file_root, "", &mut checks);
        assert!(found.is_empty());
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].severity, CheckSeverity::Error);
        assert!(checks[0].message.contains("not a directory"));

        let empty_dir = dir.join("empty");
        fs::create_dir_all(&empty_dir).unwrap();
        let mut empty_checks = Vec::new();
        let empty_found = check_recipe_search_root(&empty_dir, "", &mut empty_checks);
        assert!(empty_found.is_empty());
        assert_eq!(empty_checks.len(), 1);
        assert_eq!(empty_checks[0].severity, CheckSeverity::Warning);
        assert!(empty_checks[0].message.contains("0 .cook files"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_check_ignore_file_all_variants() {
        let dir = temp_dir("ignore_variants");

        // 1. Default missing .cookdocsignore is optional (Ok)
        let missing_default = check_ignore_file_in(&dir, ".cookdocsignore");
        assert_eq!(missing_default[0].severity, CheckSeverity::Ok);
        assert!(missing_default[0].message.contains("optional"));

        // 2. Missing custom ignore file is a Warning
        let missing_custom = check_ignore_file_in(&dir, ".myignore");
        assert_eq!(missing_custom[0].severity, CheckSeverity::Warning);
        assert!(missing_custom[0].message.contains("not found"));

        // 3. Directory instead of regular file is an Error
        let as_dir = dir.join("dir_ignore");
        fs::create_dir_all(&as_dir).unwrap();
        let dir_res = check_ignore_file_in(&dir, "dir_ignore");
        assert_eq!(dir_res[0].severity, CheckSeverity::Error);
        assert!(dir_res[0].message.contains("not a regular file"));

        // 4. Non-UTF8 ignore file fails parsing with Error
        let bad_file = dir.join(".badignore");
        fs::write(&bad_file, b"\xff\xfe\n").unwrap();
        let bad_res = check_ignore_file_in(&dir, ".badignore");
        assert_eq!(bad_res[0].severity, CheckSeverity::Error);
        assert!(bad_res[0].message.contains("Failed to parse ignore file"));

        // 5. Valid ignore file is Ok
        let good_file = dir.join(".goodignore");
        fs::write(&good_file, "drafts/\n").unwrap();
        let good_res = check_ignore_file_in(&dir, ".goodignore");
        assert_eq!(good_res[0].severity, CheckSeverity::Ok);
        assert!(good_res[0].message.contains("Ignore file valid"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_check_templates_missing_explicit_and_empty_recipes() {
        let dir = temp_dir("tpl_edges");

        let checks = check_templates(
            &dir,
            &[],
            &[
                "./missing.gotmpl".to_string(),
                "recipe.md.gotmpl".to_string(),
            ],
        );
        assert!(checks.iter().any(|c| {
            c.severity == CheckSeverity::Warning
                && c.message.contains("Configured template file not found")
        }));
        assert!(checks.iter().any(|c| {
            c.severity == CheckSeverity::Ok
                && c.message.contains("built-in/configured templates valid")
        }));

        // Broken template compilation error
        fs::write(dir.join("recipe.md.gotmpl"), "{{ .Unclosed").unwrap();
        let err_checks = check_templates(&dir, &[], &["recipe.md.gotmpl".to_string()]);
        assert!(
            err_checks
                .iter()
                .any(|c| c.severity == CheckSeverity::Error)
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_check_output_dir_all_variants() {
        let dir = temp_dir("out_variants");
        let file_path = dir.join("regular_file");
        fs::write(&file_path, "x").unwrap();

        // 1. Empty output_dir with non-directory search_root returns no checks
        assert!(check_output_dir("", &file_path).is_empty());

        // 2. Empty output_dir with writable search_root returns Ok in-place
        let in_place = check_output_dir("", &dir);
        assert_eq!(in_place[0].severity, CheckSeverity::Ok);
        assert!(in_place[0].message.contains("in-place"));

        // 3. Existing file as output_dir returns Error
        let file_out = check_output_dir(&file_path.to_string_lossy(), &dir);
        assert_eq!(file_out[0].severity, CheckSeverity::Error);
        assert!(file_out[0].message.contains("not a directory"));

        // 4. Existing writable directory returns Ok
        let existing_out = check_output_dir(&dir.to_string_lossy(), &dir);
        assert_eq!(existing_out[0].severity, CheckSeverity::Ok);
        assert!(existing_out[0].message.contains("is writable"));

        // 5. Bare non-existent relative name checks working directory "."
        let rel_out = check_output_dir("nonexistent_doctor_rel_out_xyz", &dir);
        assert_eq!(rel_out[0].severity, CheckSeverity::Ok);
        assert!(rel_out[0].message.contains("working directory is writable"));

        // 6. Non-existent output_dir whose ancestor is a regular file returns Error
        let under_file = file_path.join("nested/out");
        let bad_ancestor = check_output_dir(&under_file.to_string_lossy(), &dir);
        assert_eq!(bad_ancestor[0].severity, CheckSeverity::Error);
        assert!(bad_ancestor[0].message.contains("ancestor"));

        // 7. Read-only directory checks (Unix non-root)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let ro_dir = dir.join("readonly");
            fs::create_dir_all(&ro_dir).unwrap();
            fs::set_permissions(&ro_dir, fs::Permissions::from_mode(0o555)).unwrap();

            if check_dir_writable(&ro_dir).is_err() {
                let ro_in_place = check_output_dir("", &ro_dir);
                assert_eq!(ro_in_place[0].severity, CheckSeverity::Error);
                assert!(ro_in_place[0].message.contains("not writable"));

                let ro_existing = check_output_dir(&ro_dir.to_string_lossy(), &dir);
                assert_eq!(ro_existing[0].severity, CheckSeverity::Error);
                assert!(ro_existing[0].message.contains("not writable"));

                let ro_nested = ro_dir.join("sub/out");
                let ro_parent = check_output_dir(&ro_nested.to_string_lossy(), &dir);
                assert_eq!(ro_parent[0].severity, CheckSeverity::Error);
                assert!(ro_parent[0].message.contains("not writable"));
            }

            let _ = fs::set_permissions(&ro_dir, fs::Permissions::from_mode(0o755));
        }

        // Also cover run_doctor with explicit Some(config_path)
        let cfg = Config {
            dry_run: false,
            jsonify: false,
            ignore_file: "".to_string(),
            recipe_search_root: dir.to_string_lossy().to_string(),
            log_level: "info".to_string(),
            template_files: vec!["recipe.md.gotmpl".to_string()],
            word_wrap: 120,
            output_dir: "".to_string(),
        };
        let cfg_file = dir.join(".cookdocs.toml");
        let rep = run_doctor(Some(&cfg_file), &Ok(cfg));
        assert!(rep.healthy);
        assert!(rep.checks[0].message.contains("Configuration file valid:"));

        let _ = fs::remove_dir_all(&dir);
    }
}

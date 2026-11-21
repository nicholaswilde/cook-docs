use crate::types::Config;
use notify::event::{AccessKind, AccessMode, MetadataKind, ModifyKind};
use notify::{EventKind, RecursiveMode, Watcher};
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub const DEFAULT_DEBOUNCE_MS: u64 = 300;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchOptions {
    pub debounce: Duration,
}

impl Default for WatchOptions {
    fn default() -> Self {
        Self {
            debounce: Duration::from_millis(DEFAULT_DEBOUNCE_MS),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchTargets {
    pub search_root: PathBuf,
    pub external_dirs: Vec<PathBuf>,
    pub exact_template_paths: Vec<PathBuf>,
    pub base_template_names: Vec<String>,
}

/// Lexically collapse `.` and `..` components without touching the filesystem.
fn normalize_lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                if !matches!(
                    out.components().next_back(),
                    None | Some(Component::RootDir | Component::Prefix(_) | Component::ParentDir)
                ) {
                    out.pop();
                } else {
                    out.push(comp);
                }
            }
            _ => out.push(comp),
        }
    }
    out
}

/// Normalize a path so OS watcher events (e.g. macOS `/private/var`, Windows verbatim paths)
/// compare consistently with discovered paths even when a file does not exist yet or was deleted.
pub fn normalize_path(path: &Path) -> PathBuf {
    if let Ok(canon) = fs::canonicalize(path) {
        return canon;
    }

    let lexical = normalize_lexical(path);
    let mut suffix = Vec::new();
    let mut current = lexical.as_path();

    while !current.exists() {
        let Some(name) = current.file_name() else {
            return lexical;
        };
        suffix.push(name.to_os_string());
        match current.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => current = parent,
            _ => return lexical,
        }
    }

    let mut base = fs::canonicalize(current).unwrap_or_else(|_| current.to_path_buf());
    for part in suffix.into_iter().rev() {
        base.push(part);
    }
    base
}

fn has_git_component(path: &Path) -> bool {
    path.components()
        .any(|c| matches!(c, Component::Normal(name) if name == ".git"))
}

fn push_unique<T: Eq>(list: &mut Vec<T>, item: T) {
    if !list.contains(&item) {
        list.push(item);
    }
}

/// Compute watched directories and template matchers from search root, cwd, and configured templates.
pub fn collect_watch_targets(
    search_root: &Path,
    cwd: &Path,
    template_files: &[String],
) -> WatchTargets {
    let norm_root = normalize_path(search_root);
    let mut external_dirs = Vec::new();
    let mut exact_template_paths = Vec::new();
    let mut base_template_names = Vec::new();

    for raw in template_files {
        let tpl = raw.trim();
        if tpl.is_empty() {
            continue;
        }
        if crate::cook::is_base_filename(tpl) {
            push_unique(&mut base_template_names, tpl.to_string());
            continue;
        }

        let resolved = if crate::cook::is_relative_path(tpl) {
            search_root.join(tpl)
        } else if Path::new(tpl).is_absolute() {
            PathBuf::from(tpl)
        } else {
            cwd.join(tpl)
        };

        let norm_tpl = normalize_path(&resolved);
        if let Some(parent) = norm_tpl.parent()
            && !parent.as_os_str().is_empty()
            && !parent.starts_with(&norm_root)
        {
            push_unique(&mut external_dirs, parent.to_path_buf());
        }
        push_unique(&mut exact_template_paths, norm_tpl);
    }

    WatchTargets {
        search_root: norm_root,
        external_dirs,
        exact_template_paths,
        base_template_names,
    }
}

/// Filter out read-only and access-time-only filesystem events so reading recipes never re-triggers watch.
pub fn is_relevant_event_kind(kind: &EventKind) -> bool {
    match kind {
        EventKind::Create(_) | EventKind::Remove(_) | EventKind::Any | EventKind::Other => true,
        EventKind::Modify(ModifyKind::Metadata(MetadataKind::AccessTime)) => false,
        EventKind::Modify(_) => true,
        EventKind::Access(AccessKind::Close(AccessMode::Write)) => true,
        EventKind::Access(_) => false,
    }
}

/// Return true if a changed path could affect any recipe render (`.cook` or configured template).
/// Generated `.md` files, swap files, and `.git` paths are excluded to prevent render loops.
pub fn is_candidate_path(path: &Path, targets: &WatchTargets) -> bool {
    let norm = normalize_path(path);
    if has_git_component(&norm) {
        return false;
    }
    if targets.exact_template_paths.iter().any(|t| t == &norm) {
        return true;
    }
    if norm.starts_with(&targets.search_root) {
        if norm.extension().is_some_and(|ext| ext == "cook") {
            return true;
        }
        if let Some(name) = norm.file_name().and_then(|n| n.to_str())
            && targets.base_template_names.iter().any(|t| t == name)
        {
            return true;
        }
    }
    false
}

/// Determine which currently discovered recipes need re-rendering for a debounced batch of changed paths.
pub fn resolve_affected_recipes(
    changed_paths: &[PathBuf],
    targets: &WatchTargets,
    discovered_recipes: &[PathBuf],
) -> Vec<PathBuf> {
    let mut render_all = false;
    let mut local_template_dirs: HashSet<PathBuf> = HashSet::new();
    let mut changed_cooks: HashSet<PathBuf> = HashSet::new();

    for changed in changed_paths {
        let norm = normalize_path(changed);
        if has_git_component(&norm) {
            continue;
        }
        if targets.exact_template_paths.iter().any(|t| t == &norm) {
            render_all = true;
            break;
        }
        if norm.starts_with(&targets.search_root) {
            if let Some(name) = norm.file_name().and_then(|n| n.to_str())
                && targets.base_template_names.iter().any(|t| t == name)
                && let Some(parent) = norm.parent()
            {
                local_template_dirs.insert(parent.to_path_buf());
                continue;
            }
            if norm.extension().is_some_and(|ext| ext == "cook") {
                changed_cooks.insert(norm);
            }
        }
    }

    if render_all {
        return discovered_recipes.to_vec();
    }
    if local_template_dirs.is_empty() && changed_cooks.is_empty() {
        return Vec::new();
    }

    discovered_recipes
        .iter()
        .filter(|recipe| {
            let norm = normalize_path(recipe);
            changed_cooks.contains(&norm)
                || norm
                    .parent()
                    .is_some_and(|parent| local_template_dirs.contains(parent))
        })
        .cloned()
        .collect()
}

/// Coalesce rapid bursts of path events into deduplicated batches after `debounce` of quiet time.
pub fn process_debounced_events<F>(
    rx: &mpsc::Receiver<Option<Vec<PathBuf>>>,
    debounce: Duration,
    mut on_batch: F,
) where
    F: FnMut(Vec<PathBuf>),
{
    while let Ok(Some(first)) = rx.recv() {
        let mut pending: BTreeSet<PathBuf> = first.into_iter().collect();
        if pending.is_empty() {
            continue;
        }

        let mut stopped = false;
        if !debounce.is_zero() {
            let mut deadline = Instant::now() + debounce;
            while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
                match rx.recv_timeout(remaining) {
                    Ok(Some(paths)) => {
                        if !paths.is_empty() {
                            pending.extend(paths);
                            deadline = Instant::now() + debounce;
                        }
                    }
                    Ok(None) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                        stopped = true;
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => break,
                }
            }
        }

        on_batch(pending.into_iter().collect());

        if stopped {
            break;
        }
    }
}

/// Watch `recipe_search_root` and configured `template_files`, debouncing changes and re-rendering affected recipes.
pub fn run_watch(config: &Config, options: WatchOptions) -> Result<(), String> {
    run_watch_with_ready(config, options, |_| {})
}

fn run_watch_with_ready<F>(
    config: &Config,
    options: WatchOptions,
    on_ready: F,
) -> Result<(), String>
where
    F: FnOnce(mpsc::Sender<Option<Vec<PathBuf>>>),
{
    let search_root = crate::resolve_search_root(&config.recipe_search_root);
    if !search_root.exists() {
        return Err(format!(
            "Recipe search root '{}' does not exist",
            search_root.display()
        ));
    }
    if !search_root.is_dir() {
        return Err(format!(
            "Recipe search root '{}' is not a directory",
            search_root.display()
        ));
    }

    let search_root_str = search_root.to_string_lossy().to_string();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let targets = collect_watch_targets(&search_root, &cwd, &config.template_files);

    // 1. Initial full render
    let initial_recipes =
        crate::cook::find_recipe_file_paths(&search_root_str, &config.ignore_file);
    log::info!(
        "Found recipes [{}]",
        initial_recipes
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let (ok, failed) = crate::render_recipes(&initial_recipes, config);
    log::info!(
        "Initial render complete: {} recipe(s) rendered ({} error(s))",
        ok,
        failed
    );

    // 2. Start filesystem watcher
    let (tx, rx) = mpsc::channel::<Option<Vec<PathBuf>>>();
    let stop_tx = tx.clone();
    let callback_targets = targets.clone();
    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => {
                if !is_relevant_event_kind(&event.kind) {
                    return;
                }
                let relevant: Vec<PathBuf> = event
                    .paths
                    .into_iter()
                    .filter(|p| is_candidate_path(p, &callback_targets))
                    .collect();
                if !relevant.is_empty() {
                    let _ = tx.send(Some(relevant));
                }
            }
            Err(err) => {
                log::warn!("File watch error: {}", err);
            }
        })
        .map_err(|e| format!("Failed to initialize file watcher: {}", e))?;

    watcher
        .watch(&targets.search_root, RecursiveMode::Recursive)
        .map_err(|e| {
            format!(
                "Failed to watch recipe search root '{}': {}",
                targets.search_root.display(),
                e
            )
        })?;

    for ext_dir in &targets.external_dirs {
        if ext_dir.is_dir() {
            if let Err(e) = watcher.watch(ext_dir, RecursiveMode::NonRecursive) {
                log::warn!(
                    "Could not watch external template directory '{}': {}",
                    ext_dir.display(),
                    e
                );
            } else {
                log::debug!("Watching external template directory {}", ext_dir.display());
            }
        } else {
            log::warn!(
                "External template directory '{}' does not exist; changes there will not be watched",
                ext_dir.display()
            );
        }
    }

    log::info!(
        "Watching {} ({} recipe(s), debounce {}ms). Press Ctrl+C to stop.",
        targets.search_root.display(),
        initial_recipes.len(),
        options.debounce.as_millis()
    );

    on_ready(stop_tx);

    // 3. Debounce loop (keeps `watcher` alive until loop exits)
    process_debounced_events(&rx, options.debounce, |changed| {
        let discovered = crate::cook::find_recipe_file_paths(&search_root_str, &config.ignore_file);
        let affected = resolve_affected_recipes(&changed, &targets, &discovered);
        if affected.is_empty() {
            log::debug!("Change event matched no active recipes, skipping");
            return;
        }
        log::info!(
            "Change detected; re-rendering {} recipe(s)...",
            affected.len()
        );
        let (rendered, errors) = crate::render_recipes(&affected, config);
        log::info!("Re-rendered {} recipe(s) ({} error(s))", rendered, errors);
    });

    drop(watcher);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{
        AccessKind, AccessMode, CreateKind, DataChange, MetadataKind, ModifyKind, RemoveKind,
        RenameMode,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(prefix: &str) -> Self {
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!(
                "cook_docs_watch_{}_{}_{}",
                prefix,
                std::process::id(),
                id
            ));
            let _ = fs::remove_dir_all(&dir);
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

    fn base_config(search_root: &Path) -> Config {
        Config {
            dry_run: false,
            jsonify: false,
            ignore_file: ".cookdocsignore".to_string(),
            recipe_search_root: search_root.to_string_lossy().to_string(),
            log_level: "info".to_string(),
            template_files: vec!["recipe.md.gotmpl".to_string()],
            word_wrap: 120,
            output_dir: String::new(),
        }
    }

    #[test]
    fn test_is_relevant_event_kind() {
        assert!(is_relevant_event_kind(&EventKind::Create(CreateKind::File)));
        assert!(is_relevant_event_kind(&EventKind::Modify(
            ModifyKind::Data(DataChange::Content)
        )));
        assert!(is_relevant_event_kind(&EventKind::Modify(
            ModifyKind::Name(RenameMode::To)
        )));
        assert!(is_relevant_event_kind(&EventKind::Modify(
            ModifyKind::Metadata(MetadataKind::WriteTime)
        )));
        assert!(is_relevant_event_kind(&EventKind::Remove(RemoveKind::File)));
        assert!(is_relevant_event_kind(&EventKind::Access(
            AccessKind::Close(AccessMode::Write)
        )));

        assert!(!is_relevant_event_kind(&EventKind::Access(
            AccessKind::Open(AccessMode::Any)
        )));
        assert!(!is_relevant_event_kind(&EventKind::Access(
            AccessKind::Read
        )));
        assert!(!is_relevant_event_kind(&EventKind::Access(
            AccessKind::Close(AccessMode::Read)
        )));
        assert!(!is_relevant_event_kind(&EventKind::Modify(
            ModifyKind::Metadata(MetadataKind::AccessTime)
        )));
    }

    #[test]
    fn test_collect_watch_targets_classifies_templates_and_external_dirs() {
        let dir = TempDir::new("targets");
        let recipes_dir = dir.path().join("recipes");
        let external_dir = dir.path().join("shared_templates");
        fs::create_dir_all(&recipes_dir).unwrap();
        fs::create_dir_all(&external_dir).unwrap();

        let abs_tpl = external_dir.join("abs.gotmpl");
        fs::write(&abs_tpl, "abs").unwrap();

        let templates = vec![
            "recipe.md.gotmpl".to_string(),
            "./local.gotmpl".to_string(),
            "../shared_templates/rel.gotmpl".to_string(),
            abs_tpl.to_string_lossy().to_string(),
        ];

        let targets = collect_watch_targets(&recipes_dir, dir.path(), &templates);

        assert_eq!(targets.search_root, normalize_path(&recipes_dir));
        assert_eq!(targets.base_template_names, vec!["recipe.md.gotmpl"]);
        assert_eq!(targets.exact_template_paths.len(), 3);
        assert!(
            targets
                .exact_template_paths
                .contains(&normalize_path(&recipes_dir.join("local.gotmpl")))
        );
        assert!(
            targets
                .exact_template_paths
                .contains(&normalize_path(&external_dir.join("rel.gotmpl")))
        );
        assert!(
            targets
                .exact_template_paths
                .contains(&normalize_path(&abs_tpl))
        );
        // External dir should be deduplicated and exclude search_root itself
        assert_eq!(targets.external_dirs, vec![normalize_path(&external_dir)]);
    }

    #[test]
    fn test_resolve_affected_recipes_cook_local_and_shared_templates() {
        let dir = TempDir::new("affected");
        let root = dir.path().join("recipes");
        let soups = root.join("soups");
        let desserts = root.join("desserts");
        let shared = dir.path().join("shared");
        fs::create_dir_all(&soups).unwrap();
        fs::create_dir_all(&desserts).unwrap();
        fs::create_dir_all(&shared).unwrap();

        let tomato = soups.join("Tomato.cook");
        let miso = soups.join("Miso.cook");
        let cake = desserts.join("Cake.cook");
        let shared_tpl = shared.join("base.gotmpl");
        fs::write(&tomato, "Boil @tomato{1}.").unwrap();
        fs::write(&miso, "Boil @miso{1}.").unwrap();
        fs::write(&cake, "Bake @flour{1%cup}.").unwrap();
        fs::write(&shared_tpl, "# {{ .Info.RecipeName }}").unwrap();

        let discovered = vec![tomato.clone(), miso.clone(), cake.clone()];
        let templates = vec![
            "recipe.md.gotmpl".to_string(),
            shared_tpl.to_string_lossy().to_string(),
        ];
        let targets = collect_watch_targets(&root, dir.path(), &templates);

        // 1. Editing one .cook file only affects that recipe
        let affected =
            resolve_affected_recipes(std::slice::from_ref(&tomato), &targets, &discovered);
        assert_eq!(affected, vec![tomato.clone()]);

        // 2. Generated .md files and .git paths are ignored
        let md_file = soups.join("tomato.md");
        let git_file = root.join(".git").join("index");
        assert!(resolve_affected_recipes(&[md_file, git_file], &targets, &discovered).is_empty());

        // 3. Deleted/ignored .cook file not in discovered list is skipped
        let deleted = soups.join("Ghost.cook");
        assert!(resolve_affected_recipes(&[deleted], &targets, &discovered).is_empty());

        // 4. Local base-filename template only affects recipes in its directory
        let local_tpl = soups.join("recipe.md.gotmpl");
        let affected_local = resolve_affected_recipes(&[local_tpl], &targets, &discovered);
        assert_eq!(affected_local, vec![tomato.clone(), miso.clone()]);

        // 5. Shared external template affects all discovered recipes
        let affected_shared = resolve_affected_recipes(&[shared_tpl], &targets, &discovered);
        assert_eq!(affected_shared, discovered);
    }

    #[test]
    fn test_process_debounced_events_coalesces_burst_and_splits_batches() {
        let (tx, rx) = mpsc::channel();
        let p1 = PathBuf::from("a.cook");
        let p2 = PathBuf::from("b.cook");

        let producer = thread::spawn({
            let p1 = p1.clone();
            let p2 = p2.clone();
            move || {
                // Burst 1: three rapid events should coalesce into one batch
                tx.send(Some(vec![p1.clone()])).unwrap();
                thread::sleep(Duration::from_millis(15));
                tx.send(Some(vec![p1.clone(), p2.clone()])).unwrap();
                thread::sleep(Duration::from_millis(15));
                tx.send(Some(vec![p2.clone()])).unwrap();

                // Wait past debounce window, then send burst 2
                thread::sleep(Duration::from_millis(120));
                tx.send(Some(vec![p1.clone()])).unwrap();
            }
        });

        let mut batches = Vec::new();
        process_debounced_events(&rx, Duration::from_millis(60), |batch| {
            batches.push(batch);
        });
        producer.join().unwrap();

        assert_eq!(
            batches.len(),
            2,
            "expected 2 debounced batches: {batches:?}"
        );
        assert_eq!(batches[0], vec![p1.clone(), p2]);
        assert_eq!(batches[1], vec![p1]);
    }

    #[test]
    fn test_run_watch_missing_search_root_returns_error() {
        let missing = std::env::temp_dir().join("cook_docs_watch_nonexistent_dir_12345");
        let cfg = base_config(&missing);
        let err = run_watch(&cfg, WatchOptions::default()).unwrap_err();
        assert!(err.contains("does not exist") || err.contains("not found"));
    }

    #[test]
    fn test_run_watch_live_rerenders_recipe_and_template_changes() {
        let dir = TempDir::new("live");
        let recipe = dir.path().join("Soup.cook");
        let tpl = dir.path().join("recipe.md.gotmpl");
        let output = dir.path().join("soup.md");

        fs::write(&recipe, "Boil @water{1%l}.").unwrap();
        fs::write(
            &tpl,
            "# {{ .Info.RecipeName }}\n\n{{ template \"cook.steps\" . }}\n",
        )
        .unwrap();

        let cfg = base_config(dir.path());
        let (ready_tx, ready_rx) = mpsc::channel();

        let watch_handle = thread::spawn({
            let cfg = cfg.clone();
            move || {
                run_watch_with_ready(
                    &cfg,
                    WatchOptions {
                        debounce: Duration::from_millis(60),
                    },
                    move |stop_tx| {
                        let _ = ready_tx.send(stop_tx);
                    },
                )
            }
        });

        let stop_tx = ready_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("watcher should become ready");

        // Initial render happened on startup
        let initial = fs::read_to_string(&output).expect("initial soup.md should exist");
        assert!(initial.contains("water"));

        // Edit 1: modify .cook file
        fs::write(&recipe, "Simmer @broth{2%cups}.").unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Ok(content) = fs::read_to_string(&output)
                && content.contains("broth")
            {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "timed out waiting for recipe re-render"
            );
            thread::sleep(Duration::from_millis(25));
        }

        // Edit 2: modify .gotmpl file
        fs::write(
            &tpl,
            "# CUSTOM {{ .Info.RecipeName }}\n\n{{ template \"cook.steps\" . }}\n",
        )
        .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Ok(content) = fs::read_to_string(&output)
                && content.contains("# CUSTOM Soup")
            {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "timed out waiting for template re-render"
            );
            thread::sleep(Duration::from_millis(25));
        }

        let _ = stop_tx.send(None);
        watch_handle
            .join()
            .expect("watch thread panicked")
            .expect("run_watch failed");
    }
}

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::types::Recipe;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavEntry {
    pub title: String,
    pub rel_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavActionKind {
    Added,
    Updated,
    Deduplicated,
}

impl NavActionKind {
    pub fn verb(self) -> &'static str {
        match self {
            NavActionKind::Added => "add",
            NavActionKind::Updated => "update",
            NavActionKind::Deduplicated => "dedupe",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavChange {
    pub section: String,
    pub title: String,
    pub rel_path: String,
    pub kind: NavActionKind,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NavSyncReport {
    pub changes: Vec<NavChange>,
    pub unmatched: Vec<NavEntry>,
    pub modified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavFormat {
    Toml,
    Yaml,
}

fn detect_nav_format(path: &Path) -> Result<NavFormat, String> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("toml") => Ok(NavFormat::Toml),
        Some("yml") | Some("yaml") => Ok(NavFormat::Yaml),
        _ => Err(format!(
            "Unsupported nav file extension for '{}': expected .toml, .yml, or .yaml",
            path.display()
        )),
    }
}

/// Validate a Zensical (`zensical.toml`) or MkDocs (`mkdocs.yml`) nav file.
/// Returns the number of discovered nav sections/lists on success.
pub fn validate_nav_file(nav_file: &Path) -> Result<usize, String> {
    let format = detect_nav_format(nav_file)?;
    let content = fs::read_to_string(nav_file)
        .map_err(|e| format!("Could not read nav file '{}': {}", nav_file.display(), e))?;
    match format {
        NavFormat::Toml => {
            let doc: toml_edit::DocumentMut = content.parse().map_err(|e| {
                format!(
                    "Failed to parse TOML nav file '{}': {}",
                    nav_file.display(),
                    e
                )
            })?;
            let (sections, has_nav) = collect_toml_sections(&doc);
            if !has_nav {
                return Err(format!(
                    "Nav file '{}' does not define [project].nav or nav",
                    nav_file.display()
                ));
            }
            Ok(sections.len())
        }
        NavFormat::Yaml => {
            let parsed = parse_yaml_nav_doc(&content)
                .map_err(|e| format!("Invalid YAML nav file '{}': {}", nav_file.display(), e))?;
            let mut sections = Vec::new();
            collect_yaml_sections(&parsed.items, &mut Vec::new(), &mut sections);
            Ok(sections.len())
        }
    }
}

/// Sync rendered recipes into a Zensical (`zensical.toml`) or MkDocs (`mkdocs.yml`) nav file.
/// Respects `dry_run` by computing planned changes without writing to disk.
pub fn sync_nav_file(
    nav_file: &Path,
    recipes: &[Recipe],
    dry_run: bool,
) -> Result<NavSyncReport, String> {
    if recipes.is_empty() {
        return Ok(NavSyncReport::default());
    }

    let format = detect_nav_format(nav_file)?;
    let content = fs::read_to_string(nav_file)
        .map_err(|e| format!("Could not read nav file '{}': {}", nav_file.display(), e))?;

    let docs_dir_hint = match format {
        NavFormat::Toml => extract_toml_docs_dir(&content),
        NavFormat::Yaml => extract_yaml_docs_dir(&content),
    };

    let mut seen_paths = HashSet::new();
    let mut entries = Vec::with_capacity(recipes.len());
    for recipe in recipes {
        let entry = derive_nav_entry(recipe, nav_file, docs_dir_hint.as_deref());
        if !entry.title.is_empty()
            && !entry.rel_path.is_empty()
            && seen_paths.insert(entry.rel_path.clone())
        {
            entries.push(entry);
        }
    }
    if entries.is_empty() {
        return Ok(NavSyncReport::default());
    }

    let (updated, report) = match format {
        NavFormat::Toml => sync_toml_nav_str(&content, &entries)?,
        NavFormat::Yaml => sync_yaml_nav_str(&content, &entries)?,
    };

    if !dry_run && report.modified {
        fs::write(nav_file, updated)
            .map_err(|e| format!("Could not write nav file '{}': {}", nav_file.display(), e))?;
    }

    Ok(report)
}

/// Derive a nav entry `(Recipe Name, rel/path.md)` for a rendered recipe relative to `nav_file`.
pub fn derive_nav_entry(recipe: &Recipe, nav_file: &Path, docs_dir_hint: Option<&str>) -> NavEntry {
    let title = recipe.info.recipe_name.trim().to_string();
    let out_resolved = resolve_path_best_effort(Path::new(&recipe.info.new_recipe_file_path));

    let nav_dir = nav_file
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let docs_dir_name = docs_dir_hint
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("docs");
    let docs_root = if Path::new(docs_dir_name).is_absolute() {
        PathBuf::from(docs_dir_name)
    } else {
        nav_dir.join(docs_dir_name)
    };

    let rel_from_docs = strip_resolved_prefix(&out_resolved, &docs_root);
    let rel_from_output = if !recipe.config.output_dir.trim().is_empty() {
        strip_resolved_prefix(&out_resolved, Path::new(&recipe.config.output_dir))
    } else {
        None
    };
    let rel_from_search = strip_resolved_prefix(
        &out_resolved,
        &crate::resolve_search_root(&recipe.config.recipe_search_root),
    );

    let rel_path = rel_from_docs
        .or(rel_from_output)
        .or(rel_from_search)
        .unwrap_or_else(|| {
            let slug = if !recipe.info.recipe_slug.is_empty() {
                recipe.info.recipe_slug.clone()
            } else {
                crate::cook::slugify_filename(&recipe.info.recipe_name)
            };
            if recipe.info.categories.is_empty() {
                format!("{slug}.md")
            } else {
                format!("{}/{slug}.md", recipe.info.categories.join("/"))
            }
        });

    NavEntry {
        title,
        rel_path: normalize_rel_path(&rel_path),
    }
}

fn strip_resolved_prefix(target: &Path, base: &Path) -> Option<String> {
    let base_resolved = resolve_path_best_effort(base);
    let rel = target.strip_prefix(&base_resolved).ok()?;
    let posix = path_to_posix(rel);
    (!posix.is_empty()).then_some(posix)
}

/// Resolve a path against CWD and canonicalize its longest existing ancestor
/// so comparisons work identically on existing files, non-existent `--dry-run` outputs,
/// and symlinked temp dirs (e.g. macOS `/var` -> `/private/var`).
fn resolve_path_best_effort(path: &Path) -> PathBuf {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    let normalized = normalize_lexical(&abs);

    // Canonicalize the longest existing prefix so symlinks match.
    let mut existing = normalized.as_path();
    let mut suffix_parts = Vec::new();
    while !existing.exists() {
        if let Some(name) = existing.file_name() {
            suffix_parts.push(name.to_os_string());
        } else {
            break;
        }
        match existing.parent() {
            Some(parent) => existing = parent,
            None => break,
        }
    }

    let mut base = existing
        .canonicalize()
        .unwrap_or_else(|_| existing.to_path_buf());
    for part in suffix_parts.into_iter().rev() {
        base.push(part);
    }
    normalize_lexical(&base)
}

fn normalize_lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn path_to_posix(path: &Path) -> String {
    let parts: Vec<String> = path
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().to_string()),
            _ => None,
        })
        .collect();
    parts.join("/")
}

fn normalize_rel_path(raw: &str) -> String {
    let replaced = raw.trim().replace('\\', "/");
    let mut s = replaced.as_str();
    while let Some(stripped) = s.strip_prefix("./") {
        s = stripped;
    }
    s.trim_start_matches('/').to_string()
}

fn parent_dir(rel_path: &str) -> &str {
    match rel_path.rfind('/') {
        Some(idx) => &rel_path[..idx],
        None => "",
    }
}

fn format_section_display(display_path: &[String]) -> String {
    if display_path.is_empty() {
        "(root)".to_string()
    } else {
        display_path.join(" > ")
    }
}

fn is_index_page(rel_path: &str) -> bool {
    let norm = normalize_rel_path(rel_path).to_ascii_lowercase();
    norm == "index.md" || norm.ends_with("/index.md")
}

/// Produce slug variants for a nav section segment (e.g. "Cookies & Bars" -> ["cookies-and-bars", "cookies-bars"]).
fn segment_slugs(segment: &str) -> Vec<String> {
    let slug_with_and = slugify_segment(&segment.replace('&', " and "));
    let slug_without_and = slugify_segment(segment);
    if slug_with_and == slug_without_and {
        vec![slug_with_and]
    } else {
        vec![slug_with_and, slug_without_and]
    }
}

fn slugify_segment(raw: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in raw.chars() {
        if ch.is_alphanumeric() {
            for lc in ch.to_lowercase() {
                out.push(lc);
            }
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn section_path_matches_dir(display_path: &[String], target_dir: &str) -> bool {
    if display_path.is_empty() || target_dir.is_empty() {
        return false;
    }
    let dir_parts: Vec<&str> = target_dir.split('/').collect();
    if display_path.len() != dir_parts.len() {
        return false;
    }
    display_path
        .iter()
        .zip(dir_parts.iter())
        .all(|(seg, dir_part)| {
            let dir_lc = dir_part.to_ascii_lowercase();
            segment_slugs(seg)
                .into_iter()
                .any(|candidate| candidate == dir_lc)
        })
}

#[derive(Debug, Clone)]
struct SectionMeta {
    display_path: Vec<String>,
    leaf_paths: Vec<String>,
}

/// Find the best matching nav section index for `entry` using:
/// 0. Exact existing `rel_path` in any section (idempotent / respects manual placement)
/// 1. Root-level recipe (`dir == ""`) -> root nav list
/// 2. Exact `dir` match against direct leaves in a section
/// 3. Exact `dir` match against section title slug path (handles empty sections)
/// 4. Ancestor directory walk (longest to shortest) against direct leaves or section slugs
fn find_target_section(sections: &[SectionMeta], entry: &NavEntry) -> Option<usize> {
    let rel_path = normalize_rel_path(&entry.rel_path);
    let rel_lc = rel_path.to_ascii_lowercase();
    let dir = parent_dir(&rel_path);
    let dir_lc = dir.to_ascii_lowercase();

    // Step 0: Existing entry with exact same rel_path (prefer deepest section if duplicated)
    let mut existing_best: Option<(usize, usize)> = None;
    for (idx, sec) in sections.iter().enumerate() {
        if sec
            .leaf_paths
            .iter()
            .any(|p| p == &rel_path || p.to_ascii_lowercase() == rel_lc)
        {
            let depth = sec.display_path.len();
            if existing_best.is_none_or(|(_, best_depth)| depth > best_depth) {
                existing_best = Some((idx, depth));
            }
        }
    }
    if let Some((idx, _)) = existing_best {
        return Some(idx);
    }

    // Step 1: Root-level recipe (no subdirectory)
    if dir.is_empty() {
        return sections.iter().position(|s| s.display_path.is_empty());
    }

    // Step 2: Exact directory match on direct leaves (prefer deepest section, then most matches)
    let mut exact_leaf_best: Option<(usize, usize, usize)> = None;
    for (idx, sec) in sections.iter().enumerate() {
        if sec.display_path.is_empty() {
            continue;
        }
        let matches = sec
            .leaf_paths
            .iter()
            .filter(|p| parent_dir(p).to_ascii_lowercase() == dir_lc)
            .count();
        if matches > 0 {
            let depth = sec.display_path.len();
            if exact_leaf_best
                .is_none_or(|(_, best_depth, best_cnt)| (depth, matches) > (best_depth, best_cnt))
            {
                exact_leaf_best = Some((idx, depth, matches));
            }
        }
    }
    if let Some((idx, _, _)) = exact_leaf_best {
        return Some(idx);
    }

    // Step 3: Exact directory match against section title slug path
    if let Some(idx) = sections
        .iter()
        .position(|s| section_path_matches_dir(&s.display_path, &dir_lc))
    {
        return Some(idx);
    }

    // Step 4: Ancestor walk from longest to shortest
    let mut ancestor = dir;
    loop {
        let ancestor_lc = ancestor.to_ascii_lowercase();
        let ancestor_prefix = format!("{ancestor_lc}/");
        let ancestor_depth = ancestor.split('/').count();

        // 4a: Direct leaves in `ancestor` OR section at depth <= ancestor_depth with leaves under `ancestor/`
        let mut ancestor_leaf_best: Option<(usize, usize, usize)> = None;
        for (idx, sec) in sections.iter().enumerate() {
            if sec.display_path.is_empty() {
                continue;
            }
            let direct_matches = sec
                .leaf_paths
                .iter()
                .filter(|p| parent_dir(p).to_ascii_lowercase() == ancestor_lc)
                .count();
            let prefix_matches = if sec.display_path.len() <= ancestor_depth {
                sec.leaf_paths
                    .iter()
                    .filter(|p| p.to_ascii_lowercase().starts_with(&ancestor_prefix))
                    .count()
            } else {
                0
            };
            let score = direct_matches * 10 + prefix_matches;
            if score > 0 {
                let depth = sec.display_path.len();
                if ancestor_leaf_best.is_none_or(|(_, best_depth, best_score)| {
                    (depth, score) > (best_depth, best_score)
                }) {
                    ancestor_leaf_best = Some((idx, depth, score));
                }
            }
        }
        if let Some((idx, _, _)) = ancestor_leaf_best {
            return Some(idx);
        }

        // 4b: Section slug path matches `ancestor`
        if let Some(idx) = sections
            .iter()
            .position(|s| section_path_matches_dir(&s.display_path, &ancestor_lc))
        {
            return Some(idx);
        }

        match ancestor.rfind('/') {
            Some(pos) => ancestor = &ancestor[..pos],
            None => break,
        }
    }

    None
}

// =============================================================================
// TOML (`zensical.toml`) implementation using `toml_edit`
// =============================================================================

#[derive(Debug, Clone)]
enum TomlNavRoot {
    ProjectArrayOfTables {
        table_idx: usize,
        key: String,
    },
    ProjectInlineArray,
    TopArrayOfTables {
        table_idx: usize,
        key: String,
    },
    TopInlineArray,
    /// Root `[[project.nav]]` or `[[nav]]` array-of-tables container itself (for root-level recipes)
    ProjectArrayOfTablesRoot,
    TopArrayOfTablesRoot,
}

#[derive(Debug, Clone)]
struct TomlSection {
    meta: SectionMeta,
    root: TomlNavRoot,
    nested_keys: Vec<String>,
}

fn extract_toml_docs_dir(content: &str) -> Option<String> {
    let doc: toml_edit::DocumentMut = content.parse().ok()?;
    doc.get("project")
        .and_then(|p| p.get("docs_dir"))
        .and_then(|v| v.as_str())
        .or_else(|| doc.get("docs_dir").and_then(|v| v.as_str()))
        .map(ToString::to_string)
}

fn collect_toml_sections(doc: &toml_edit::DocumentMut) -> (Vec<TomlSection>, bool) {
    let mut sections = Vec::new();

    if let Some(project) = doc.get("project").and_then(|item| item.as_table_like())
        && let Some(nav_item) = project.get("nav")
    {
        collect_toml_nav_item(nav_item, true, &mut sections);
        return (sections, true);
    }

    if let Some(nav_item) = doc.get("nav") {
        collect_toml_nav_item(nav_item, false, &mut sections);
        return (sections, true);
    }

    (sections, false)
}

fn collect_toml_nav_item(
    nav_item: &toml_edit::Item,
    under_project: bool,
    sections: &mut Vec<TomlSection>,
) {
    match nav_item {
        toml_edit::Item::ArrayOfTables(aot) => {
            let mut root_leaf_paths = Vec::new();
            for (table_idx, table) in aot.iter().enumerate() {
                for (key, item) in table.iter() {
                    if let Some(path) = item.as_str() {
                        root_leaf_paths.push(normalize_rel_path(path));
                    } else if let Some(arr) = item.as_array() {
                        let root = if under_project {
                            TomlNavRoot::ProjectArrayOfTables {
                                table_idx,
                                key: key.to_string(),
                            }
                        } else {
                            TomlNavRoot::TopArrayOfTables {
                                table_idx,
                                key: key.to_string(),
                            }
                        };
                        collect_toml_array_sections(arr, &root, &[key.to_string()], &[], sections);
                    }
                }
            }
            sections.push(TomlSection {
                meta: SectionMeta {
                    display_path: Vec::new(),
                    leaf_paths: root_leaf_paths,
                },
                root: if under_project {
                    TomlNavRoot::ProjectArrayOfTablesRoot
                } else {
                    TomlNavRoot::TopArrayOfTablesRoot
                },
                nested_keys: Vec::new(),
            });
        }
        toml_edit::Item::Value(toml_edit::Value::Array(arr)) => {
            let root = if under_project {
                TomlNavRoot::ProjectInlineArray
            } else {
                TomlNavRoot::TopInlineArray
            };
            collect_toml_array_sections(arr, &root, &[], &[], sections);
        }
        _ => {}
    }
}

fn collect_toml_array_sections(
    arr: &toml_edit::Array,
    root: &TomlNavRoot,
    display_path: &[String],
    nested_keys: &[String],
    sections: &mut Vec<TomlSection>,
) {
    let mut direct_leaves = Vec::new();
    let mut sub_arrays: Vec<(String, &toml_edit::Array)> = Vec::new();

    for val in arr.iter() {
        match val {
            toml_edit::Value::String(s) => {
                direct_leaves.push(normalize_rel_path(s.value()));
            }
            toml_edit::Value::InlineTable(tbl) if tbl.len() == 1 => {
                if let Some((k, v)) = tbl.iter().next() {
                    match v {
                        toml_edit::Value::String(s) => {
                            direct_leaves.push(normalize_rel_path(s.value()));
                        }
                        toml_edit::Value::Array(sub) => {
                            sub_arrays.push((k.to_string(), sub));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    sections.push(TomlSection {
        meta: SectionMeta {
            display_path: display_path.to_vec(),
            leaf_paths: direct_leaves,
        },
        root: root.clone(),
        nested_keys: nested_keys.to_vec(),
    });

    for (sub_key, sub_arr) in sub_arrays {
        let mut next_display = display_path.to_vec();
        next_display.push(sub_key.clone());
        let mut next_nested = nested_keys.to_vec();
        next_nested.push(sub_key);
        collect_toml_array_sections(sub_arr, root, &next_display, &next_nested, sections);
    }
}

fn resolve_toml_array_mut<'a>(
    doc: &'a mut toml_edit::DocumentMut,
    sec: &TomlSection,
) -> Option<&'a mut toml_edit::Array> {
    let mut arr = match &sec.root {
        TomlNavRoot::ProjectArrayOfTables { table_idx, key } => doc
            .get_mut("project")?
            .as_table_like_mut()?
            .get_mut("nav")?
            .as_array_of_tables_mut()?
            .get_mut(*table_idx)?
            .get_mut(key)?
            .as_array_mut()?,
        TomlNavRoot::TopArrayOfTables { table_idx, key } => doc
            .get_mut("nav")?
            .as_array_of_tables_mut()?
            .get_mut(*table_idx)?
            .get_mut(key)?
            .as_array_mut()?,
        TomlNavRoot::ProjectInlineArray => doc
            .get_mut("project")?
            .as_table_like_mut()?
            .get_mut("nav")?
            .as_array_mut()?,
        TomlNavRoot::TopInlineArray => doc.get_mut("nav")?.as_array_mut()?,
        TomlNavRoot::ProjectArrayOfTablesRoot | TomlNavRoot::TopArrayOfTablesRoot => {
            return None;
        }
    };

    for key in &sec.nested_keys {
        let mut next_arr = None;
        for val in arr.iter_mut() {
            if let toml_edit::Value::InlineTable(tbl) = val
                && tbl.len() == 1
                && tbl.contains_key(key)
                && let Some(sub) = tbl.get_mut(key).and_then(|v| v.as_array_mut())
            {
                next_arr = Some(sub);
                break;
            }
        }
        arr = next_arr?;
    }

    Some(arr)
}

fn detect_toml_indent_unit(content: &str) -> usize {
    for line in content.lines() {
        let trimmed = line.trim_start_matches(' ');
        if trimmed.starts_with('{') {
            let spaces = line.len() - trimmed.len();
            if spaces > 0 {
                return spaces;
            }
        }
    }
    2
}

fn detect_toml_array_indent(
    arr: &toml_edit::Array,
    depth: usize,
    indent_unit: usize,
) -> (String, String) {
    for val in arr.iter() {
        if let Some(prefix) = val.decor().prefix().and_then(|p| p.as_str())
            && let Some(nl) = prefix.rfind('\n')
        {
            let elem_indent = prefix[nl + 1..].to_string();
            let closing_indent = arr
                .trailing()
                .as_str()
                .and_then(|t| t.rfind('\n').map(|pos| t[pos + 1..].to_string()))
                .unwrap_or_else(|| " ".repeat(elem_indent.len().saturating_sub(indent_unit)));
            return (elem_indent, closing_indent);
        }
    }
    let elem_indent = " ".repeat((depth + 1) * indent_unit);
    let closing_indent = " ".repeat(depth * indent_unit);
    (elem_indent, closing_indent)
}

fn build_toml_leaf_table(title: &str, rel_path: &str, elem_indent: &str) -> toml_edit::Value {
    let raw = format!(
        "{{ {} = {} }}",
        toml_edit::Value::from(title),
        toml_edit::Value::from(rel_path)
    );
    let mut val = raw.parse::<toml_edit::Value>().unwrap_or_else(|_| {
        let mut tbl = toml_edit::InlineTable::new();
        tbl.insert(title, toml_edit::Value::from(rel_path));
        toml_edit::Value::InlineTable(tbl)
    });
    val.decor_mut().set_prefix(format!("\n{elem_indent}"));
    val.decor_mut().set_suffix("");
    val
}

struct TomlLeafItem {
    title: String,
    rel_path: String,
    value: toml_edit::Value,
}

fn extract_toml_leaf(val: &toml_edit::Value) -> Option<(String, String)> {
    if let toml_edit::Value::InlineTable(tbl) = val
        && tbl.len() == 1
        && let Some((k, toml_edit::Value::String(s))) = tbl.iter().next()
    {
        return Some((k.to_string(), normalize_rel_path(s.value())));
    }
    None
}

fn apply_entries_to_toml_array(
    arr: &mut toml_edit::Array,
    section_name: &str,
    entries: &[NavEntry],
    depth: usize,
    indent_unit: usize,
    changes: &mut Vec<NavChange>,
) -> bool {
    let was_empty = arr.is_empty();
    let (elem_indent, closing_indent) = detect_toml_array_indent(arr, depth, indent_unit);
    let preserve_trailing_comma = if was_empty {
        true
    } else {
        arr.trailing_comma()
    };

    let mut pinned: Vec<toml_edit::Value> = Vec::new();
    let mut leaves: Vec<TomlLeafItem> = Vec::new();
    let mut original_leaf_order: Vec<(String, String)> = Vec::new();
    let mut saw_change = false;

    for val in arr.iter() {
        if let Some((title, rel_path)) = extract_toml_leaf(val) {
            if is_index_page(&rel_path) {
                pinned.push(val.clone());
                continue;
            }
            original_leaf_order.push((title.clone(), rel_path.clone()));
            let title_lc = title.to_ascii_lowercase();
            let path_lc = rel_path.to_ascii_lowercase();
            let dup = leaves.iter().any(|existing| {
                existing.rel_path.to_ascii_lowercase() == path_lc
                    || existing.title.to_ascii_lowercase() == title_lc
            });
            if dup {
                saw_change = true;
                changes.push(NavChange {
                    section: section_name.to_string(),
                    title,
                    rel_path,
                    kind: NavActionKind::Deduplicated,
                });
            } else {
                let mut cloned = val.clone();
                if cloned
                    .decor()
                    .prefix()
                    .and_then(|p| p.as_str())
                    .is_none_or(|p| !p.contains('\n'))
                {
                    cloned.decor_mut().set_prefix(format!("\n{elem_indent}"));
                }
                leaves.push(TomlLeafItem {
                    title,
                    rel_path,
                    value: cloned,
                });
            }
        } else {
            pinned.push(val.clone());
        }
    }

    for entry in entries {
        let norm_path = normalize_rel_path(&entry.rel_path);
        let path_lc = norm_path.to_ascii_lowercase();
        let title_lc = entry.title.to_ascii_lowercase();

        // Check existing by rel_path first, then by case-insensitive title.
        let pos_by_path = leaves
            .iter()
            .position(|l| l.rel_path.to_ascii_lowercase() == path_lc);
        let pos_by_title = leaves
            .iter()
            .position(|l| l.title.to_ascii_lowercase() == title_lc);

        if let Some(idx) = pos_by_path {
            if leaves[idx].title != entry.title || leaves[idx].rel_path != norm_path {
                leaves[idx].title = entry.title.clone();
                leaves[idx].rel_path = norm_path.clone();
                leaves[idx].value = build_toml_leaf_table(&entry.title, &norm_path, &elem_indent);
                saw_change = true;
                changes.push(NavChange {
                    section: section_name.to_string(),
                    title: entry.title.clone(),
                    rel_path: norm_path,
                    kind: NavActionKind::Updated,
                });
            }
            // If another leaf had the same title (different path), remove duplicate
            if let Some(t_idx) = pos_by_title
                && t_idx != idx
            {
                let removed = leaves.remove(t_idx);
                saw_change = true;
                changes.push(NavChange {
                    section: section_name.to_string(),
                    title: removed.title,
                    rel_path: removed.rel_path,
                    kind: NavActionKind::Deduplicated,
                });
            }
        } else if let Some(idx) = pos_by_title {
            if leaves[idx].title != entry.title || leaves[idx].rel_path != norm_path {
                leaves[idx].title = entry.title.clone();
                leaves[idx].rel_path = norm_path.clone();
                leaves[idx].value = build_toml_leaf_table(&entry.title, &norm_path, &elem_indent);
                saw_change = true;
                changes.push(NavChange {
                    section: section_name.to_string(),
                    title: entry.title.clone(),
                    rel_path: norm_path,
                    kind: NavActionKind::Updated,
                });
            }
        } else {
            leaves.push(TomlLeafItem {
                title: entry.title.clone(),
                rel_path: norm_path.clone(),
                value: build_toml_leaf_table(&entry.title, &norm_path, &elem_indent),
            });
            saw_change = true;
            changes.push(NavChange {
                section: section_name.to_string(),
                title: entry.title.clone(),
                rel_path: norm_path,
                kind: NavActionKind::Added,
            });
        }
    }

    leaves.sort_by(|a, b| {
        a.title
            .to_lowercase()
            .cmp(&b.title.to_lowercase())
            .then_with(|| a.title.cmp(&b.title))
            .then_with(|| a.rel_path.cmp(&b.rel_path))
    });

    let new_leaf_order: Vec<(String, String)> = leaves
        .iter()
        .map(|l| (l.title.clone(), l.rel_path.clone()))
        .collect();
    if !saw_change && new_leaf_order == original_leaf_order {
        return false;
    }

    arr.clear();
    for val in pinned {
        arr.push_formatted(val);
    }
    for leaf in leaves {
        arr.push_formatted(leaf.value);
    }
    arr.set_trailing_comma(preserve_trailing_comma);
    if was_empty || arr.trailing().as_str().is_none_or(|t| !t.contains('\n')) {
        arr.set_trailing(format!("\n{closing_indent}"));
    }

    true
}

fn apply_entries_to_toml_root_aot(
    doc: &mut toml_edit::DocumentMut,
    under_project: bool,
    entries: &[NavEntry],
    changes: &mut Vec<NavChange>,
) -> bool {
    let aot = if under_project {
        doc.get_mut("project")
            .and_then(|p| p.as_table_like_mut())
            .and_then(|p| p.get_mut("nav"))
            .and_then(|n| n.as_array_of_tables_mut())
    } else {
        doc.get_mut("nav").and_then(|n| n.as_array_of_tables_mut())
    };
    let Some(aot) = aot else {
        return false;
    };

    let mut modified = false;
    for entry in entries {
        let norm_path = normalize_rel_path(&entry.rel_path);
        let path_lc = norm_path.to_ascii_lowercase();
        let title_lc = entry.title.to_ascii_lowercase();

        let mut matched_idx = None;
        for (i, table) in aot.iter().enumerate() {
            if table.len() == 1
                && let Some((k, item)) = table.iter().next()
                && let Some(existing_path) = item.as_str()
            {
                let ep_norm = normalize_rel_path(existing_path);
                if ep_norm.to_ascii_lowercase() == path_lc || k.to_ascii_lowercase() == title_lc {
                    matched_idx = Some((i, k.to_string(), ep_norm));
                    break;
                }
            }
        }

        if let Some((idx, existing_key, existing_path)) = matched_idx {
            if existing_key != entry.title || existing_path != norm_path {
                if let Some(table) = aot.get_mut(idx) {
                    table.remove(&existing_key);
                    table.insert(&entry.title, toml_edit::value(norm_path.clone()));
                }
                modified = true;
                changes.push(NavChange {
                    section: "(root)".to_string(),
                    title: entry.title.clone(),
                    rel_path: norm_path,
                    kind: NavActionKind::Updated,
                });
            }
        } else {
            let mut table = toml_edit::Table::new();
            table.insert(&entry.title, toml_edit::value(norm_path.clone()));
            aot.push(table);
            modified = true;
            changes.push(NavChange {
                section: "(root)".to_string(),
                title: entry.title.clone(),
                rel_path: norm_path,
                kind: NavActionKind::Added,
            });
        }
    }
    modified
}

/// Sync `entries` into a Zensical TOML document string while preserving formatting & comments.
pub fn sync_toml_nav_str(
    content: &str,
    entries: &[NavEntry],
) -> Result<(String, NavSyncReport), String> {
    let mut doc: toml_edit::DocumentMut = content
        .parse()
        .map_err(|e| format!("Failed to parse TOML nav file: {e}"))?;

    let (sections, has_nav) = collect_toml_sections(&doc);
    if !has_nav {
        return Err("TOML nav file does not define [project].nav or nav".to_string());
    }

    let metas: Vec<SectionMeta> = sections.iter().map(|s| s.meta.clone()).collect();
    let mut grouped: BTreeMap<usize, Vec<NavEntry>> = BTreeMap::new();
    let mut report = NavSyncReport::default();

    for entry in entries {
        match find_target_section(&metas, entry) {
            Some(sec_idx) => grouped.entry(sec_idx).or_default().push(entry.clone()),
            None => report.unmatched.push(entry.clone()),
        }
    }

    let indent_unit = detect_toml_indent_unit(content);
    let mut modified = false;

    for (sec_idx, sec_entries) in grouped {
        let sec = &sections[sec_idx];
        let section_name = format_section_display(&sec.meta.display_path);
        match sec.root {
            TomlNavRoot::ProjectArrayOfTablesRoot => {
                if apply_entries_to_toml_root_aot(&mut doc, true, &sec_entries, &mut report.changes)
                {
                    modified = true;
                }
            }
            TomlNavRoot::TopArrayOfTablesRoot => {
                if apply_entries_to_toml_root_aot(
                    &mut doc,
                    false,
                    &sec_entries,
                    &mut report.changes,
                ) {
                    modified = true;
                }
            }
            _ => {
                let depth = match sec.root {
                    TomlNavRoot::ProjectArrayOfTables { .. }
                    | TomlNavRoot::TopArrayOfTables { .. } => sec.nested_keys.len(),
                    TomlNavRoot::ProjectInlineArray | TomlNavRoot::TopInlineArray => {
                        sec.nested_keys.len()
                    }
                    _ => 0,
                };
                if let Some(arr) = resolve_toml_array_mut(&mut doc, sec)
                    && apply_entries_to_toml_array(
                        arr,
                        &section_name,
                        &sec_entries,
                        depth,
                        indent_unit,
                        &mut report.changes,
                    )
                {
                    modified = true;
                }
            }
        }
    }

    report.modified = modified;
    if modified {
        Ok((doc.to_string(), report))
    } else {
        Ok((content.to_string(), report))
    }
}

// =============================================================================
// YAML (`mkdocs.yml` / `mkdocs.yaml`) comment- and tag-preserving implementation
// =============================================================================

#[derive(Debug, Clone)]
enum YamlNavItem {
    Raw(String),
    BareLeaf {
        indent: usize,
        path: String,
        raw_line: String,
    },
    Leaf {
        indent: usize,
        title: String,
        path: String,
        raw_line: String,
    },
    Section {
        indent: usize,
        title: String,
        header_line: String,
        was_inline_empty: bool,
        children: Vec<YamlNavItem>,
    },
}

struct ParsedYamlNavDoc {
    before_nav: Vec<String>,
    nav_header_line: String,
    nav_was_inline_empty: bool,
    items: Vec<YamlNavItem>,
    after_nav: Vec<String>,
    line_ending: &'static str,
    trailing_newline: bool,
    indent_step: usize,
}

fn extract_yaml_docs_dir(content: &str) -> Option<String> {
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("docs_dir:") {
            let cleaned = strip_yaml_inline_comment(rest).trim();
            if !cleaned.is_empty() {
                return Some(unquote_yaml_scalar(cleaned));
            }
        }
    }
    None
}

fn leading_spaces(line: &str) -> usize {
    line.bytes().take_while(|&b| b == b' ').count()
}

fn is_blank_or_comment(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.is_empty() || trimmed.starts_with('#')
}

fn strip_yaml_inline_comment(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if in_single {
            if b == b'\'' {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                    i += 2;
                    continue;
                }
                in_single = false;
            }
        } else if in_double {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == b'"' {
                in_double = false;
            }
        } else {
            match b {
                b'\'' => in_single = true,
                b'"' => in_double = true,
                b'#' if i == 0 || bytes[i - 1].is_ascii_whitespace() => {
                    return &s[..i];
                }
                _ => {}
            }
        }
        i += 1;
    }
    s
}

fn unquote_yaml_scalar(raw: &str) -> String {
    let s = raw.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        let inner = &s[1..s.len() - 1];
        let mut out = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some(other) => {
                        out.push('\\');
                        out.push(other);
                    }
                    None => out.push('\\'),
                }
            } else {
                out.push(c);
            }
        }
        return out;
    }
    if s.len() >= 2 && s.starts_with('\'') && s.ends_with('\'') {
        return s[1..s.len() - 1].replace("''", "'");
    }
    s.to_string()
}

fn needs_yaml_quotes(s: &str) -> bool {
    if s.is_empty() || s.trim() != s {
        return true;
    }
    let lc = s.to_ascii_lowercase();
    if matches!(
        lc.as_str(),
        "true" | "false" | "null" | "~" | "yes" | "no" | "on" | "off"
    ) {
        return true;
    }
    if let Some(first) = s.chars().next()
        && matches!(
            first,
            '-' | '?'
                | ':'
                | ','
                | '['
                | ']'
                | '{'
                | '}'
                | '#'
                | '&'
                | '*'
                | '!'
                | '|'
                | '>'
                | '\''
                | '"'
                | '%'
                | '@'
                | '`'
        )
    {
        return true;
    }
    s.contains(": ")
        || s.ends_with(':')
        || s.contains(" #")
        || s.contains('"')
        || s.contains('\\')
        || s.contains('\n')
        || s.contains('\r')
        || s.contains('\t')
}

fn format_yaml_scalar(s: &str, force_double_quotes: bool) -> String {
    if force_double_quotes || needs_yaml_quotes(s) {
        let escaped = s
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

/// Split a YAML mapping item after `- ` into `(raw_key, Option<raw_value>)`.
/// Returns `None` if there is no unquoted key-value separator `:` (i.e. bare leaf path).
fn split_yaml_mapping(after_dash: &str) -> Option<(String, Option<String>)> {
    let no_comment = strip_yaml_inline_comment(after_dash).trim();
    if no_comment.is_empty() {
        return None;
    }

    let bytes = no_comment.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;
    let mut colon_idx = None;
    while i < bytes.len() {
        let b = bytes[i];
        if in_single {
            if b == b'\'' {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                    i += 2;
                    continue;
                }
                in_single = false;
            }
        } else if in_double {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == b'"' {
                in_double = false;
            }
        } else {
            match b {
                b'\'' => in_single = true,
                b'"' => in_double = true,
                b':' if i + 1 == bytes.len() || bytes[i + 1].is_ascii_whitespace() => {
                    colon_idx = Some(i);
                    break;
                }
                _ => {}
            }
        }
        i += 1;
    }

    let pos = colon_idx?;
    let key = unquote_yaml_scalar(&no_comment[..pos]);
    let rest = no_comment[pos + 1..].trim();
    if rest.is_empty() {
        Some((key, None))
    } else {
        Some((key, Some(rest.to_string())))
    }
}

fn parse_yaml_nav_doc(content: &str) -> Result<ParsedYamlNavDoc, String> {
    let line_ending = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let trailing_newline = content.ends_with('\n');
    let lines: Vec<String> = content
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l).to_string())
        .collect();
    let lines = if trailing_newline && lines.last().is_some_and(String::is_empty) {
        lines[..lines.len() - 1].to_vec()
    } else {
        lines
    };

    let nav_idx = lines
        .iter()
        .position(|line| {
            if let Some(rest) = line.strip_prefix("nav:") {
                let cleaned = strip_yaml_inline_comment(rest).trim();
                cleaned.is_empty() || cleaned == "[]"
            } else {
                false
            }
        })
        .ok_or_else(|| "YAML nav file does not define a top-level 'nav:' key".to_string())?;

    let nav_header_line = lines[nav_idx].clone();
    let nav_rest = strip_yaml_inline_comment(nav_header_line.strip_prefix("nav:").unwrap()).trim();
    let nav_was_inline_empty = nav_rest == "[]";

    let mut nav_end = nav_idx + 1;
    while nav_end < lines.len() {
        let line = &lines[nav_end];
        if !is_blank_or_comment(line) {
            let indent = leading_spaces(line);
            let trimmed = line.trim_start();
            if indent == 0 && !trimmed.starts_with("- ") && trimmed != "-" {
                break;
            }
        }
        nav_end += 1;
    }

    // Keep trailing blank lines / column-0 comments outside the nav block (before next top-level key)
    let mut trimmed_nav_end = nav_end;
    while trimmed_nav_end > nav_idx + 1 {
        let line = &lines[trimmed_nav_end - 1];
        if line.trim().is_empty() || line.starts_with('#') {
            trimmed_nav_end -= 1;
        } else {
            break;
        }
    }

    let before_nav = lines[..nav_idx].to_vec();
    let nav_body = &lines[nav_idx + 1..trimmed_nav_end];
    let after_nav = lines[trimmed_nav_end..].to_vec();

    let (items, _) = parse_yaml_list(nav_body, 0, None);
    let indent_step = detect_yaml_indent_step(&items).unwrap_or(2);

    Ok(ParsedYamlNavDoc {
        before_nav,
        nav_header_line,
        nav_was_inline_empty,
        items,
        after_nav,
        line_ending,
        trailing_newline,
        indent_step,
    })
}

fn parse_yaml_list(
    lines: &[String],
    mut idx: usize,
    expected_indent: Option<usize>,
) -> (Vec<YamlNavItem>, usize) {
    let mut items = Vec::new();
    let mut list_indent = expected_indent;

    while idx < lines.len() {
        let line = &lines[idx];
        if is_blank_or_comment(line) {
            // Only keep blank/comment lines inside this list if they belong at >= list_indent
            // and a sibling at >= list_indent follows.
            if let Some(li) = list_indent {
                if !line.trim().is_empty() && leading_spaces(line) < li {
                    break;
                }
                let next_non_blank =
                    (idx + 1..lines.len()).find(|&j| !is_blank_or_comment(&lines[j]));
                if let Some(next_idx) = next_non_blank
                    && leading_spaces(&lines[next_idx]) < li
                {
                    break;
                }
            }
            items.push(YamlNavItem::Raw(line.clone()));
            idx += 1;
            continue;
        }

        let indent = leading_spaces(line);
        let trimmed = &line[indent..];

        if let Some(li) = list_indent {
            if indent < li {
                break;
            }
            if indent > li {
                items.push(YamlNavItem::Raw(line.clone()));
                idx += 1;
                continue;
            }
        } else {
            if !trimmed.starts_with("- ") {
                items.push(YamlNavItem::Raw(line.clone()));
                idx += 1;
                continue;
            }
            list_indent = Some(indent);
        }

        let Some(after_dash) = trimmed.strip_prefix("- ") else {
            items.push(YamlNavItem::Raw(line.clone()));
            idx += 1;
            continue;
        };

        match split_yaml_mapping(after_dash) {
            None => {
                let raw_val = strip_yaml_inline_comment(after_dash).trim();
                items.push(YamlNavItem::BareLeaf {
                    indent,
                    path: normalize_rel_path(&unquote_yaml_scalar(raw_val)),
                    raw_line: line.clone(),
                });
                idx += 1;
            }
            Some((title, Some(raw_val))) if raw_val == "[]" => {
                items.push(YamlNavItem::Section {
                    indent,
                    title,
                    header_line: line.clone(),
                    was_inline_empty: true,
                    children: Vec::new(),
                });
                idx += 1;
            }
            Some((title, Some(raw_val))) => {
                items.push(YamlNavItem::Leaf {
                    indent,
                    title,
                    path: normalize_rel_path(&unquote_yaml_scalar(&raw_val)),
                    raw_line: line.clone(),
                });
                idx += 1;
            }
            Some((title, None)) => {
                let header_line = line.clone();
                idx += 1;
                // Find child block end: all following lines that belong to indent > `indent`
                let next_non_blank = (idx..lines.len()).find(|&j| !is_blank_or_comment(&lines[j]));
                let children = if let Some(first_child) = next_non_blank
                    && leading_spaces(&lines[first_child]) > indent
                {
                    let child_indent = leading_spaces(&lines[first_child]);
                    let (parsed_children, next_idx) =
                        parse_yaml_list(lines, idx, Some(child_indent));
                    idx = next_idx;
                    parsed_children
                } else {
                    Vec::new()
                };
                items.push(YamlNavItem::Section {
                    indent,
                    title,
                    header_line,
                    was_inline_empty: false,
                    children,
                });
            }
        }
    }

    (items, idx)
}

fn detect_yaml_indent_step(items: &[YamlNavItem]) -> Option<usize> {
    for item in items {
        if let YamlNavItem::Section {
            indent, children, ..
        } = item
        {
            for child in children {
                let child_indent = match child {
                    YamlNavItem::Leaf { indent, .. }
                    | YamlNavItem::BareLeaf { indent, .. }
                    | YamlNavItem::Section { indent, .. } => Some(*indent),
                    YamlNavItem::Raw(_) => None,
                };
                if let Some(ci) = child_indent
                    && ci > *indent
                {
                    return Some(ci - *indent);
                }
            }
            if let Some(step) = detect_yaml_indent_step(children) {
                return Some(step);
            }
        }
    }
    None
}

fn collect_yaml_sections(
    items: &[YamlNavItem],
    display_path: &mut Vec<String>,
    out: &mut Vec<SectionMeta>,
) {
    let mut direct_leaves = Vec::new();
    for item in items {
        match item {
            YamlNavItem::Leaf { path, .. } | YamlNavItem::BareLeaf { path, .. } => {
                direct_leaves.push(path.clone());
            }
            _ => {}
        }
    }
    out.push(SectionMeta {
        display_path: display_path.clone(),
        leaf_paths: direct_leaves,
    });

    for item in items {
        if let YamlNavItem::Section {
            title, children, ..
        } = item
        {
            display_path.push(title.clone());
            collect_yaml_sections(children, display_path, out);
            display_path.pop();
        }
    }
}

fn yaml_list_uses_quoted_keys(items: &[YamlNavItem]) -> bool {
    let mut quoted = 0usize;
    let mut unquoted = 0usize;
    for item in items {
        if let YamlNavItem::Leaf { raw_line, .. } = item {
            let trimmed = raw_line.trim_start();
            if let Some(after_dash) = trimmed.strip_prefix("- ") {
                if after_dash.starts_with('"') {
                    quoted += 1;
                } else {
                    unquoted += 1;
                }
            }
        }
    }
    quoted > 0 && unquoted == 0
}

fn yaml_list_uses_quoted_paths(items: &[YamlNavItem]) -> bool {
    let mut quoted = 0usize;
    let mut unquoted = 0usize;
    for item in items {
        if let YamlNavItem::Leaf { raw_line, .. } = item
            && let Some(after_dash) = raw_line.trim_start().strip_prefix("- ")
            && let Some((_, Some(raw_val))) = split_yaml_mapping(after_dash)
        {
            if raw_val.starts_with('"') {
                quoted += 1;
            } else {
                unquoted += 1;
            }
        }
    }
    quoted > 0 && unquoted == 0
}

fn format_yaml_leaf_line(
    indent: usize,
    title: &str,
    rel_path: &str,
    quote_key: bool,
    quote_path: bool,
) -> String {
    format!(
        "{}- {}: {}",
        " ".repeat(indent),
        format_yaml_scalar(title, quote_key),
        format_yaml_scalar(rel_path, quote_path)
    )
}

fn apply_entries_to_yaml_list(
    items: &mut Vec<YamlNavItem>,
    section_name: &str,
    entries: &[NavEntry],
    default_child_indent: usize,
    changes: &mut Vec<NavChange>,
) -> bool {
    let child_indent = items
        .iter()
        .find_map(|item| match item {
            YamlNavItem::Leaf { indent, .. }
            | YamlNavItem::BareLeaf { indent, .. }
            | YamlNavItem::Section { indent, .. } => Some(*indent),
            YamlNavItem::Raw(_) => None,
        })
        .unwrap_or(default_child_indent);

    let quote_key = yaml_list_uses_quoted_keys(items);
    let quote_path = yaml_list_uses_quoted_paths(items);

    let mut pinned: Vec<YamlNavItem> = Vec::new();
    let mut leaves: Vec<(String, String, String)> = Vec::new();
    let mut original_leaf_order: Vec<(String, String)> = Vec::new();
    let mut saw_change = false;

    for item in items.drain(..) {
        match item {
            YamlNavItem::Leaf {
                title,
                path,
                raw_line,
                ..
            } => {
                if is_index_page(&path) {
                    pinned.push(YamlNavItem::Leaf {
                        indent: child_indent,
                        title,
                        path,
                        raw_line,
                    });
                    continue;
                }
                original_leaf_order.push((title.clone(), path.clone()));
                let title_lc = title.to_ascii_lowercase();
                let path_lc = path.to_ascii_lowercase();
                let dup = leaves.iter().any(|(et, ep, _)| {
                    ep.to_ascii_lowercase() == path_lc || et.to_ascii_lowercase() == title_lc
                });
                if dup {
                    saw_change = true;
                    changes.push(NavChange {
                        section: section_name.to_string(),
                        title,
                        rel_path: path,
                        kind: NavActionKind::Deduplicated,
                    });
                } else {
                    leaves.push((title, path, raw_line));
                }
            }
            other => pinned.push(other),
        }
    }

    for entry in entries {
        let norm_path = normalize_rel_path(&entry.rel_path);
        let path_lc = norm_path.to_ascii_lowercase();
        let title_lc = entry.title.to_ascii_lowercase();

        let pos_by_path = leaves
            .iter()
            .position(|(_, ep, _)| ep.to_ascii_lowercase() == path_lc);
        let pos_by_title = leaves
            .iter()
            .position(|(et, _, _)| et.to_ascii_lowercase() == title_lc);

        if let Some(idx) = pos_by_path {
            if leaves[idx].0 != entry.title || leaves[idx].1 != norm_path {
                leaves[idx] = (
                    entry.title.clone(),
                    norm_path.clone(),
                    format_yaml_leaf_line(
                        child_indent,
                        &entry.title,
                        &norm_path,
                        quote_key,
                        quote_path,
                    ),
                );
                saw_change = true;
                changes.push(NavChange {
                    section: section_name.to_string(),
                    title: entry.title.clone(),
                    rel_path: norm_path,
                    kind: NavActionKind::Updated,
                });
            }
            if let Some(t_idx) = pos_by_title
                && t_idx != idx
            {
                let (rt, rp, _) = leaves.remove(t_idx);
                saw_change = true;
                changes.push(NavChange {
                    section: section_name.to_string(),
                    title: rt,
                    rel_path: rp,
                    kind: NavActionKind::Deduplicated,
                });
            }
        } else if let Some(idx) = pos_by_title {
            if leaves[idx].0 != entry.title || leaves[idx].1 != norm_path {
                leaves[idx] = (
                    entry.title.clone(),
                    norm_path.clone(),
                    format_yaml_leaf_line(
                        child_indent,
                        &entry.title,
                        &norm_path,
                        quote_key,
                        quote_path,
                    ),
                );
                saw_change = true;
                changes.push(NavChange {
                    section: section_name.to_string(),
                    title: entry.title.clone(),
                    rel_path: norm_path,
                    kind: NavActionKind::Updated,
                });
            }
        } else {
            leaves.push((
                entry.title.clone(),
                norm_path.clone(),
                format_yaml_leaf_line(
                    child_indent,
                    &entry.title,
                    &norm_path,
                    quote_key,
                    quote_path,
                ),
            ));
            saw_change = true;
            changes.push(NavChange {
                section: section_name.to_string(),
                title: entry.title.clone(),
                rel_path: norm_path,
                kind: NavActionKind::Added,
            });
        }
    }

    leaves.sort_by(|a, b| {
        a.0.to_lowercase()
            .cmp(&b.0.to_lowercase())
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| a.1.cmp(&b.1))
    });

    let new_leaf_order: Vec<(String, String)> = leaves
        .iter()
        .map(|(t, p, _)| (t.clone(), p.clone()))
        .collect();
    let changed = saw_change || new_leaf_order != original_leaf_order;

    items.extend(pinned);
    for (title, path, raw_line) in leaves {
        items.push(YamlNavItem::Leaf {
            indent: child_indent,
            title,
            path,
            raw_line,
        });
    }

    changed
}

fn apply_yaml_updates_recursive(
    items: &mut Vec<YamlNavItem>,
    target_path: &[String],
    entries: &[NavEntry],
    parent_indent: Option<usize>,
    indent_step: usize,
    changes: &mut Vec<NavChange>,
) -> bool {
    if target_path.is_empty() {
        let default_indent = parent_indent
            .map(|i| i + indent_step)
            .unwrap_or(indent_step);
        return apply_entries_to_yaml_list(items, "(root)", entries, default_indent, changes);
    }

    let head = &target_path[0];
    let tail = &target_path[1..];
    for item in items.iter_mut() {
        if let YamlNavItem::Section {
            indent,
            title,
            header_line,
            was_inline_empty,
            children,
        } = item
            && title == head
        {
            let modified = if tail.is_empty() {
                let default_child_indent = *indent + indent_step;
                let section_name = target_path.join(" > ");
                let ch = apply_entries_to_yaml_list(
                    children,
                    &section_name,
                    entries,
                    default_child_indent,
                    changes,
                );
                if ch && *was_inline_empty && !children.is_empty() {
                    *header_line = strip_inline_empty_brackets(header_line);
                    *was_inline_empty = false;
                }
                ch
            } else {
                // Pass full display path via leaf call below
                apply_yaml_updates_with_full_name(
                    children,
                    tail,
                    target_path,
                    *indent,
                    indent_step,
                    entries,
                    changes,
                )
            };
            return modified;
        }
    }
    false
}

fn apply_yaml_updates_with_full_name(
    items: &mut Vec<YamlNavItem>,
    remaining_path: &[String],
    full_path: &[String],
    parent_indent: usize,
    indent_step: usize,
    entries: &[NavEntry],
    changes: &mut Vec<NavChange>,
) -> bool {
    if remaining_path.is_empty() {
        return apply_entries_to_yaml_list(
            items,
            &full_path.join(" > "),
            entries,
            parent_indent + indent_step,
            changes,
        );
    }
    let head = &remaining_path[0];
    let tail = &remaining_path[1..];
    for item in items.iter_mut() {
        if let YamlNavItem::Section {
            indent,
            title,
            header_line,
            was_inline_empty,
            children,
        } = item
            && title == head
        {
            if tail.is_empty() {
                let ch = apply_entries_to_yaml_list(
                    children,
                    &full_path.join(" > "),
                    entries,
                    *indent + indent_step,
                    changes,
                );
                if ch && *was_inline_empty && !children.is_empty() {
                    *header_line = strip_inline_empty_brackets(header_line);
                    *was_inline_empty = false;
                }
                return ch;
            }
            return apply_yaml_updates_with_full_name(
                children,
                tail,
                full_path,
                *indent,
                indent_step,
                entries,
                changes,
            );
        }
    }
    false
}

fn strip_inline_empty_brackets(header_line: &str) -> String {
    if let Some(pos) = header_line.find(": []") {
        let mut out = String::with_capacity(header_line.len());
        out.push_str(&header_line[..pos + 1]);
        out.push_str(&header_line[pos + 4..]);
        out.trim_end().to_string()
    } else {
        header_line.to_string()
    }
}

fn render_yaml_items(items: &[YamlNavItem], out: &mut Vec<String>) {
    for item in items {
        match item {
            YamlNavItem::Raw(line)
            | YamlNavItem::BareLeaf { raw_line: line, .. }
            | YamlNavItem::Leaf { raw_line: line, .. } => out.push(line.clone()),
            YamlNavItem::Section {
                header_line,
                children,
                ..
            } => {
                out.push(header_line.clone());
                render_yaml_items(children, out);
            }
        }
    }
}

/// Sync `entries` into a MkDocs YAML document string while preserving comments, formatting,
/// and non-standard YAML tags (e.g. `!!python/name:...`) outside `nav:`.
pub fn sync_yaml_nav_str(
    content: &str,
    entries: &[NavEntry],
) -> Result<(String, NavSyncReport), String> {
    let mut parsed = parse_yaml_nav_doc(content)?;
    let mut sections = Vec::new();
    collect_yaml_sections(&parsed.items, &mut Vec::new(), &mut sections);

    let mut grouped: BTreeMap<usize, Vec<NavEntry>> = BTreeMap::new();
    let mut report = NavSyncReport::default();

    for entry in entries {
        match find_target_section(&sections, entry) {
            Some(sec_idx) => grouped.entry(sec_idx).or_default().push(entry.clone()),
            None => report.unmatched.push(entry.clone()),
        }
    }

    let mut modified = false;
    for (sec_idx, sec_entries) in grouped {
        let target_path = &sections[sec_idx].display_path;
        if apply_yaml_updates_recursive(
            &mut parsed.items,
            target_path,
            &sec_entries,
            None,
            parsed.indent_step,
            &mut report.changes,
        ) {
            modified = true;
        }
    }

    report.modified = modified;
    if !modified {
        return Ok((content.to_string(), report));
    }

    if parsed.nav_was_inline_empty && !parsed.items.is_empty() {
        parsed.nav_header_line = strip_inline_empty_brackets(&parsed.nav_header_line);
    }

    let mut out_lines = Vec::new();
    out_lines.extend(parsed.before_nav);
    out_lines.push(parsed.nav_header_line);
    render_yaml_items(&parsed.items, &mut out_lines);
    out_lines.extend(parsed.after_nav);

    let mut rendered = out_lines.join(parsed.line_ending);
    if parsed.trailing_newline {
        rendered.push_str(parsed.line_ending);
    }

    Ok((rendered, report))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_array_of_tables_insert_sort_and_preserve_comments() {
        let input = r#"[project]
site_name = "Recipes"

# Top navigation comment
[[project.nav]]
Home = "index.md"

[[project.nav]]
"Cookies & Bars" = [
  { "Almond Biscotti" = "cookies-and-bars/almond-biscotti.md" },
  { "Snickerdoodles" = "cookies-and-bars/snickerdoodles.md" },
]

[project.theme]
name = "material"
"#;

        let entries = vec![NavEntry {
            title: "Date Brownies".to_string(),
            rel_path: "cookies-and-bars/date-brownies.md".to_string(),
        }];

        let (out, report) = sync_toml_nav_str(input, &entries).unwrap();
        assert!(report.modified);
        assert!(report.unmatched.is_empty());
        assert_eq!(report.changes.len(), 1);
        assert_eq!(report.changes[0].kind, NavActionKind::Added);
        assert_eq!(report.changes[0].section, "Cookies & Bars");

        let expected = r#"[project]
site_name = "Recipes"

# Top navigation comment
[[project.nav]]
Home = "index.md"

[[project.nav]]
"Cookies & Bars" = [
  { "Almond Biscotti" = "cookies-and-bars/almond-biscotti.md" },
  { "Date Brownies" = "cookies-and-bars/date-brownies.md" },
  { "Snickerdoodles" = "cookies-and-bars/snickerdoodles.md" },
]

[project.theme]
name = "material"
"#;
        assert_eq!(out, expected);

        // Idempotent second run
        let (out2, report2) = sync_toml_nav_str(&out, &entries).unwrap();
        assert!(!report2.modified);
        assert!(report2.changes.is_empty());
        assert_eq!(out2, out);
    }

    #[test]
    fn test_toml_nested_subcategories_and_flattened_parent() {
        let input = r#"[project]
site_name = "Recipes"

[[project.nav]]
Desserts = [
  { "Apple Cake" = "desserts/cake/apple-cake.md" },
  { "Banana Pudding" = "desserts/banana-pudding.md" },
]

[[project.nav]]
Ingredients = [
  { "Butter" = [
    { "Almond Butter" = "ingredients/butter/almond-butter.md" },
    { "Ghee" = "ingredients/butter/ghee.md" },
  ] },
  { "Pastry Dough" = [] },
  { "Apple Butter" = "ingredients/apple-butter.md" },
  { "Yogurt" = "ingredients/yogurt.md" },
]
"#;

        let entries = vec![
            NavEntry {
                title: "Clarified Butter".to_string(),
                rel_path: "ingredients/butter/clarified-butter.md".to_string(),
            },
            NavEntry {
                title: "All-Butter Pie Crust".to_string(),
                rel_path: "ingredients/pastry-dough/all-butter-pie-crust.md".to_string(),
            },
            NavEntry {
                title: "Buttermilk".to_string(),
                rel_path: "ingredients/buttermilk.md".to_string(),
            },
            NavEntry {
                title: "Key Lime Pie".to_string(),
                rel_path: "desserts/pie/key-lime-pie.md".to_string(),
            },
        ];

        let (out, report) = sync_toml_nav_str(input, &entries).unwrap();
        assert!(report.modified);
        assert!(report.unmatched.is_empty());
        assert_eq!(report.changes.len(), 4);

        // Verify exact placements and preserved sub-sections
        assert!(out.contains(
            "  { \"Butter\" = [\n    { \"Almond Butter\" = \"ingredients/butter/almond-butter.md\" },\n    { \"Clarified Butter\" = \"ingredients/butter/clarified-butter.md\" },\n    { \"Ghee\" = \"ingredients/butter/ghee.md\" },\n  ] },"
        ));
        assert!(out.contains(
            "  { \"Pastry Dough\" = [\n    { \"All-Butter Pie Crust\" = \"ingredients/pastry-dough/all-butter-pie-crust.md\" },\n  ] },"
        ));
        let idx_butter_sec = out.find("{ \"Butter\" = [").unwrap();
        let idx_apple = out.find("{ \"Apple Butter\"").unwrap();
        let idx_buttermilk = out.find("{ \"Buttermilk\"").unwrap();
        let idx_yogurt = out.find("{ \"Yogurt\"").unwrap();
        assert!(
            idx_butter_sec < idx_apple && idx_apple < idx_buttermilk && idx_buttermilk < idx_yogurt
        );
        assert!(out.contains("{ \"Key Lime Pie\" = \"desserts/pie/key-lime-pie.md\" },"));

        // Must remain valid TOML
        assert!(out.parse::<toml_edit::DocumentMut>().is_ok());
    }

    #[test]
    fn test_toml_inline_project_nav_and_deduplication() {
        let input = r#"[project]
site_name = "Recipes"
nav = [
    { "Home" = "index.md" },
    { "Breads" = [
        { "Sourdough" = "breads/sourdough.md" },
        { "Bagels" = "breads/bagels.md" },
        { "Bagels" = "breads/bagels.md" },
    ] },
]
"#;

        let entries = vec![NavEntry {
            title: "Brioche".to_string(),
            rel_path: "breads/brioche.md".to_string(),
        }];

        let (out, report) = sync_toml_nav_str(input, &entries).unwrap();
        assert!(report.modified);
        assert_eq!(out.matches("breads/bagels.md").count(), 1);
        assert!(
            report
                .changes
                .iter()
                .any(|c| c.kind == NavActionKind::Deduplicated && c.title == "Bagels")
        );
        assert!(
            report
                .changes
                .iter()
                .any(|c| c.kind == NavActionKind::Added && c.title == "Brioche")
        );
        let idx_bagels = out.find("Bagels").unwrap();
        let idx_brioche = out.find("Brioche").unwrap();
        let idx_sourdough = out.find("Sourdough").unwrap();
        assert!(idx_bagels < idx_brioche && idx_brioche < idx_sourdough);
    }

    #[test]
    fn test_yaml_mkdocs_nested_comments_python_tags_dedupe_and_idempotency() {
        let input = r#"site_name: My Recipes
markdown_extensions:
  - pymdownx.emoji:
      emoji_index: !!python/name:material.extensions.emoji.twemoji

# Main Navigation
nav:
  - Home: index.md
  - Breads:
      - Sourdough: breads/sourdough.md
      - Bagels: breads/bagels.md
      - Bagels: breads/bagels.md
  - Ingredients:
      - Butter:
          - Almond Butter: ingredients/butter/almond-butter.md
          - Ghee: ingredients/butter/ghee.md
      - Pastry Dough: []
      - Apple Butter: ingredients/apple-butter.md

# Theme settings
theme:
  name: material
"#;

        let entries = vec![
            NavEntry {
                title: "Brioche".to_string(),
                rel_path: "breads/brioche.md".to_string(),
            },
            NavEntry {
                title: "Clarified Butter".to_string(),
                rel_path: "ingredients/butter/clarified-butter.md".to_string(),
            },
            NavEntry {
                title: "Pie Crust: Flaky".to_string(),
                rel_path: "ingredients/pastry-dough/pie-crust-flaky.md".to_string(),
            },
            NavEntry {
                title: "Buttermilk".to_string(),
                rel_path: "ingredients/buttermilk.md".to_string(),
            },
        ];

        let (out, report) = sync_yaml_nav_str(input, &entries).unwrap();
        assert!(report.modified);
        assert!(report.unmatched.is_empty());
        assert!(out.contains("!!python/name:material.extensions.emoji.twemoji"));
        assert!(out.contains("# Main Navigation"));
        assert!(out.contains("# Theme settings"));
        assert_eq!(out.matches("breads/bagels.md").count(), 1);
        assert!(out.contains("      - Bagels: breads/bagels.md\n      - Brioche: breads/brioche.md\n      - Sourdough: breads/sourdough.md"));
        assert!(out.contains("          - Almond Butter: ingredients/butter/almond-butter.md\n          - Clarified Butter: ingredients/butter/clarified-butter.md\n          - Ghee: ingredients/butter/ghee.md"));
        assert!(out.contains("      - Pastry Dough:\n          - \"Pie Crust: Flaky\": ingredients/pastry-dough/pie-crust-flaky.md"));
        assert!(out.contains("      - Apple Butter: ingredients/apple-butter.md\n      - Buttermilk: ingredients/buttermilk.md"));

        // Second run is completely idempotent
        let (out2, report2) = sync_yaml_nav_str(&out, &entries).unwrap();
        assert!(!report2.modified);
        assert!(report2.changes.is_empty());
        assert_eq!(out2, out);
    }

    #[test]
    fn test_sibling_subcategory_does_not_steal_unknown_subcategory() {
        let input = r#"[project]
site_name = "Recipes"

[[project.nav]]
Ingredients = [
  { "Butter" = [
    { "Almond Butter" = "ingredients/butter/almond-butter.md" },
  ] },
]
"#;
        let entries = vec![NavEntry {
            title: "Ricotta".to_string(),
            rel_path: "ingredients/cheese/ricotta.md".to_string(),
        }];
        let (out, report) = sync_toml_nav_str(input, &entries).unwrap();
        assert!(report.modified);
        assert_eq!(report.changes[0].section, "Ingredients");
        assert!(out.contains("  { \"Ricotta\" = \"ingredients/cheese/ricotta.md\" },"));
    }

    #[test]
    fn test_unmatched_category_is_reported_without_modifying_file() {
        let input = r#"[project]
site_name = "Recipes"

[[project.nav]]
Breads = [
  { "Bagels" = "breads/bagels.md" },
]
"#;
        let entries = vec![NavEntry {
            title: "Popcorn".to_string(),
            rel_path: "snacks/popcorn.md".to_string(),
        }];
        let (out, report) = sync_toml_nav_str(input, &entries).unwrap();
        assert!(!report.modified);
        assert_eq!(report.unmatched.len(), 1);
        assert_eq!(report.unmatched[0].title, "Popcorn");
        assert_eq!(out, input);
    }

    #[test]
    fn test_sync_nav_file_dry_run_does_not_modify_disk() {
        let dir = std::env::temp_dir().join(format!(
            "cook_docs_nav_dry_run_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(dir.join("docs/breads")).unwrap();
        let nav_path = dir.join("zensical.toml");
        let original = r#"[project]
site_name = "Test"

[[project.nav]]
Breads = [
  { "Bagels" = "breads/bagels.md" },
]
"#;
        fs::write(&nav_path, original).unwrap();

        let mut recipe = Recipe {
            steps: Vec::new(),
            ingredients: Vec::new(),
            cookware: Vec::new(),
            metadata: std::collections::HashMap::new(),
            frontmatter: BTreeMap::new(),
            config: crate::types::Config {
                output_dir: dir.join("docs").to_string_lossy().to_string(),
                nav_file: nav_path.to_string_lossy().to_string(),
                dry_run: true,
                ..crate::types::Config::default()
            },
            info: crate::types::Info {
                image_file_name: String::new(),
                image_file_path: String::new(),
                image_slug: String::new(),
                new_recipe_file_path: dir
                    .join("docs/breads/brioche.md")
                    .to_string_lossy()
                    .to_string(),
                recipe_name: "Brioche".to_string(),
                recipe_slug: "brioche".to_string(),
                recipe_file_path: dir
                    .join("cook/breads/Brioche.cook")
                    .to_string_lossy()
                    .to_string(),
                rel_root: "..".to_string(),
                categories: vec!["breads".to_string()],
                default_tags: Vec::new(),
                tags: Vec::new(),
            },
        };

        let dry_report = sync_nav_file(&nav_path, std::slice::from_ref(&recipe), true).unwrap();
        assert!(dry_report.modified);
        assert_eq!(dry_report.changes.len(), 1);
        assert_eq!(fs::read_to_string(&nav_path).unwrap(), original);

        recipe.config.dry_run = false;
        let real_report = sync_nav_file(&nav_path, std::slice::from_ref(&recipe), false).unwrap();
        assert!(real_report.modified);
        let updated = fs::read_to_string(&nav_path).unwrap();
        assert!(updated.contains("{ \"Brioche\" = \"breads/brioche.md\" },"));

        let _ = fs::remove_dir_all(&dir);
    }
}

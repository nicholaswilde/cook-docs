use figment::{
    Figment,
    providers::{Format, Yaml},
};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, HashMap, HashSet};

pub const BEGIN_MARKER: &str = "<!-- cook-docs:begin -->";
pub const END_MARKER: &str = "<!-- cook-docs:end -->";

/// Non-destructively merge freshly rendered Markdown (`compiled`) into an existing Markdown file
/// (`existing`), preserving manual edits.
///
/// Supports two modes automatically:
/// 1. **Region markers**: if `existing` contains `<!-- cook-docs:begin -->` and
///    `<!-- cook-docs:end -->`, replaces only the content between those markers.
/// 2. **Structure-aware merge**: merges YAML frontmatter (unioning `tags`, keeping manual keys
///    and `hero`), preserves H1 emoji/image embeds, preserves custom `##` sections in relative
///    order, preserves manual `=== "..."` tabs in Ingredients/Instructions, merges `Source`
///    bullets, preserves manual admonitions (`!!!`/`???`) and ```` ```mermaid ```` diagrams, and
///    preserves footer reference/footnote definitions.
pub fn merge_markdown(existing: &str, compiled: &str) -> String {
    let existing_norm = existing.replace("\r\n", "\n");
    let compiled_norm = compiled.replace("\r\n", "\n");

    if let Some(merged) = merge_marker_region(&existing_norm, &compiled_norm) {
        return merged;
    }

    merge_structured(&existing_norm, &compiled_norm)
}

fn merge_marker_region(existing: &str, compiled: &str) -> Option<String> {
    let begin_idx = existing.find(BEGIN_MARKER)?;
    let after_begin = begin_idx + BEGIN_MARKER.len();
    let end_rel = existing[after_begin..].find(END_MARKER)?;
    let end_idx = after_begin + end_rel;

    // If compiled output also contains markers, extract only its inner region to stay idempotent
    let inner_compiled = if let Some(cb) = compiled.find(BEGIN_MARKER)
        && let Some(ce_rel) = compiled[cb + BEGIN_MARKER.len()..].find(END_MARKER)
    {
        let start = cb + BEGIN_MARKER.len();
        compiled[start..start + ce_rel].trim_matches('\n')
    } else {
        compiled.trim_matches('\n')
    };

    let prefix = &existing[..after_begin];
    let suffix = &existing[end_idx..];

    let mut out = String::with_capacity(existing.len() + inner_compiled.len());
    out.push_str(prefix);
    out.push('\n');
    if !inner_compiled.is_empty() {
        out.push_str(inner_compiled);
        out.push('\n');
    }
    out.push_str(suffix);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}

fn split_frontmatter(content: &str) -> (Option<JsonMap<String, JsonValue>>, &str) {
    let Some(rest) = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))
    else {
        return (None, content);
    };

    let mut offset = 0usize;
    for line in rest.split_inclusive('\n') {
        if line.trim() == "---" {
            let yaml_text = &rest[..offset];
            let body = rest[offset + line.len()..].trim_start_matches('\n');
            if yaml_text.trim().is_empty() {
                return (Some(JsonMap::new()), body);
            }
            if let Ok(JsonValue::Object(map)) = Figment::new()
                .merge(Yaml::string(yaml_text))
                .extract::<JsonValue>()
            {
                return (Some(map), body);
            }
            return (None, content);
        }
        offset += line.len();
    }

    (None, content)
}

fn merge_frontmatter(
    existing: Option<JsonMap<String, JsonValue>>,
    compiled: Option<JsonMap<String, JsonValue>>,
) -> Option<BTreeMap<String, JsonValue>> {
    if existing.is_none() && compiled.is_none() {
        return None;
    }
    let mut merged: BTreeMap<String, JsonValue> = BTreeMap::new();
    if let Some(ex) = existing {
        for (k, v) in ex {
            merged.insert(k, v);
        }
    }
    if let Some(co) = compiled {
        for (k, v) in co {
            if k == "tags" {
                let mut combined = match merged.get("tags") {
                    Some(JsonValue::Array(arr)) => arr.clone(),
                    _ => Vec::new(),
                };
                if let JsonValue::Array(new_tags) = v {
                    for tag in new_tags {
                        if !combined.contains(&tag) {
                            combined.push(tag);
                        }
                    }
                }
                merged.insert(k, JsonValue::Array(combined));
            } else if k == "hero" {
                let compiled_empty = match &v {
                    JsonValue::Null => true,
                    JsonValue::String(s) => s.trim().is_empty(),
                    _ => false,
                };
                if !compiled_empty || !merged.contains_key("hero") {
                    merged.insert(k, v);
                }
            } else {
                merged.insert(k, v);
            }
        }
    }
    Some(merged)
}

fn format_frontmatter_yaml(map: &BTreeMap<String, JsonValue>) -> String {
    if map.is_empty() {
        return "---\n---\n\n".to_string();
    }
    let gtmpl_map: HashMap<String, gtmpl_ng::Value> = map
        .iter()
        .map(|(k, v)| (k.clone(), crate::document::json_to_gtmpl_value(v.clone())))
        .collect();
    let body = crate::document::format_yaml_from_gtmpl(&gtmpl_ng::Value::Map(gtmpl_map));
    format!("---\n{body}\n---\n\n")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StandardSection {
    Ingredients,
    Cookware,
    Instructions,
    Source,
    Metadata,
    Comments,
}

fn classify_h2(heading: &str) -> Option<StandardSection> {
    let low = heading.to_ascii_lowercase();
    if low.contains("ingredients") {
        Some(StandardSection::Ingredients)
    } else if low.contains("cookware") {
        Some(StandardSection::Cookware)
    } else if low.contains("instructions") || low.contains("steps") {
        Some(StandardSection::Instructions)
    } else if low.contains("source") {
        Some(StandardSection::Source)
    } else if low.contains("metadata") {
        Some(StandardSection::Metadata)
    } else if low.contains("comments") {
        Some(StandardSection::Comments)
    } else {
        None
    }
}

fn is_ref_def(line: &str) -> Option<String> {
    let trimmed = line.trim_end();
    if !trimmed.starts_with('[') || trimmed.starts_with("[^") {
        return None;
    }
    let close = trimmed.find("]:")?;
    let label = &trimmed[..=close];
    if label.len() <= 2 || label.contains('\n') {
        return None;
    }
    let after = &trimmed[close + 2..];
    if after.is_empty() || !after.starts_with(char::is_whitespace) {
        return None;
    }
    Some(label.to_string())
}

fn is_footnote_def(line: &str) -> Option<String> {
    if !line.starts_with("[^") {
        return None;
    }
    let close = line.find("]:")?;
    let label = &line[..=close];
    (label.len() > 3).then(|| label.to_string())
}

fn extract_footer_defs(lines: &[String]) -> (Vec<String>, Vec<(String, String)>) {
    let mut defs = Vec::new();
    let mut keep = Vec::new();
    let mut i = 0;
    let mut in_fence = false;

    while i < lines.len() {
        let line = &lines[i];
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            keep.push(line.clone());
            i += 1;
            continue;
        }
        if !in_fence {
            if let Some(label) = is_footnote_def(line) {
                let mut block = vec![line.clone()];
                i += 1;
                while i < lines.len() {
                    let nxt = &lines[i];
                    let is_continuation = nxt.starts_with("    ")
                        || nxt.starts_with('\t')
                        || (nxt.trim().is_empty()
                            && i + 1 < lines.len()
                            && (lines[i + 1].starts_with("    ")
                                || lines[i + 1].starts_with('\t')));
                    if is_continuation {
                        block.push(nxt.clone());
                        i += 1;
                    } else {
                        break;
                    }
                }
                defs.push((label, block.join("\n").trim_end().to_string()));
                continue;
            }
            if let Some(label) = is_ref_def(line) {
                defs.push((label, line.trim().to_string()));
                i += 1;
                continue;
            }
        }
        keep.push(line.clone());
        i += 1;
    }

    while keep.last().is_some_and(|l| l.trim().is_empty()) {
        keep.pop();
    }
    (keep, defs)
}

fn split_h2_sections(lines: &[String]) -> (Vec<String>, Vec<(String, Vec<String>)>) {
    let mut preamble = Vec::new();
    let mut sections = Vec::new();
    let mut cur_heading: Option<String> = None;
    let mut cur_lines: Vec<String> = Vec::new();
    let mut in_fence = false;

    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
        }
        if !in_fence && line.starts_with("## ") {
            if let Some(h) = cur_heading.take() {
                sections.push((h, std::mem::take(&mut cur_lines)));
            } else {
                preamble = std::mem::take(&mut cur_lines);
            }
            cur_heading = Some(line.trim().to_string());
        } else {
            cur_lines.push(line.clone());
        }
    }

    if let Some(h) = cur_heading {
        sections.push((h, cur_lines));
    } else {
        preamble = cur_lines;
    }

    (preamble, sections)
}

fn parse_h1(line: &str) -> Option<(Option<&str>, &str)> {
    let rest = line.trim().strip_prefix("# ")?.trim();
    if let Some(after_colon) = rest.strip_prefix(':')
        && let Some(end_colon) = after_colon.find(':')
        && end_colon > 0
    {
        let token = &rest[..=end_colon + 1];
        let valid_shortcode = token[1..token.len() - 1]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+'));
        if valid_shortcode {
            let title = rest[end_colon + 2..].trim();
            return Some((Some(token), title));
        }
    }
    Some((None, rest))
}

fn is_image_embed(line: &str) -> bool {
    let s = line.trim();
    if !s.starts_with("![") {
        return false;
    }
    let Some(alt_end) = s.find(']') else {
        return false;
    };
    let rest = &s[alt_end + 1..];
    rest.starts_with('(') || rest.starts_with('[')
}

fn extract_image_ref_key(line: &str) -> Option<String> {
    let s = line.trim();
    if !s.starts_with("![") {
        return None;
    }
    let alt_end = s.find(']')?;
    let rest = &s[alt_end + 1..];
    if let Some(after_open) = rest.strip_prefix('[') {
        let close = after_open.find(']')?;
        if close > 0 {
            return Some(format!("[{}]", &after_open[..close]));
        }
        let alt = &s[2..alt_end];
        if !alt.is_empty() {
            return Some(format!("[{alt}]"));
        }
    }
    None
}

fn body_references_key(lines: &[String], key: &str) -> bool {
    let key_lower = key.to_ascii_lowercase();
    if key_lower.len() <= 2 {
        return false;
    }
    let mut in_fence = false;
    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let lower = line.to_ascii_lowercase();
        let mut search_from = 0;
        while let Some(rel) = lower[search_from..].find(&key_lower) {
            let start = search_from + rel;
            let after = &lower[start + key_lower.len()..];
            if !after.starts_with('(') && !after.starts_with(':') {
                return true;
            }
            search_from = start + key_lower.len();
        }
    }
    false
}

fn extract_urls(text: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut idx = 0;
    while idx < text.len() {
        let slice = &text[idx..];
        let next_pos = slice
            .find("https://")
            .into_iter()
            .chain(slice.find("http://"))
            .min();
        let Some(pos) = next_pos else {
            break;
        };
        let start = idx + pos;
        let tail = &text[start..];
        let end_rel = tail
            .find(|c: char| c.is_whitespace() || matches!(c, ')' | '>' | '"' | '\''))
            .unwrap_or(tail.len());
        let url = tail[..end_rel].trim_end_matches('/').to_string();
        if !url.is_empty() {
            urls.push(url);
        }
        idx = start + end_rel.max(1);
    }
    urls
}

fn merge_source_lines(
    existing: &[String],
    compiled: &[String],
    existing_defs: &[(String, String)],
) -> Vec<String> {
    let existing_bullets: Vec<String> = existing
        .iter()
        .map(|l| l.trim().to_string())
        .filter(|l| l.starts_with("- "))
        .collect();
    let compiled_bullets: Vec<String> = compiled
        .iter()
        .map(|l| l.trim().to_string())
        .filter(|l| l.starts_with("- "))
        .collect();

    if existing_bullets.is_empty() {
        return compiled.to_vec();
    }

    let mut seen_urls: HashSet<String> = HashSet::new();
    let mut seen_texts: HashSet<String> = HashSet::new();
    for b in &existing_bullets {
        seen_texts.insert(b.to_ascii_lowercase());
        for url in extract_urls(b) {
            seen_urls.insert(url);
        }
        // Resolve reference-style links like `- [Title][1]` against existing footer defs
        for (ref_label, def_block) in existing_defs {
            if b.contains(ref_label.as_str()) {
                for url in extract_urls(def_block) {
                    seen_urls.insert(url);
                }
            }
        }
    }

    let mut merged = existing_bullets;
    for b in compiled_bullets {
        let urls = extract_urls(&b);
        if !urls.is_empty() && urls.iter().any(|u| seen_urls.contains(u)) {
            continue;
        }
        if seen_texts.contains(&b.to_ascii_lowercase()) {
            continue;
        }
        merged.push(b);
    }

    let mut out = Vec::with_capacity(merged.len() + 2);
    out.push(String::new());
    out.extend(merged);
    out.push(String::new());
    out
}

fn is_admonition_start(line: &str) -> bool {
    line.starts_with("!!! ") || line.starts_with("??? ") || line.starts_with("???+ ")
}

fn is_mermaid_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    if let Some(after) = trimmed.strip_prefix("```") {
        let lang = after.split_whitespace().next().unwrap_or("");
        return lang.eq_ignore_ascii_case("mermaid");
    }
    false
}

fn extract_manual_blocks(lines: &[String]) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut current_heading = "__top__".to_string();
    let mut i = 0;

    while i < lines.len() {
        let line = &lines[i];
        if line.starts_with("## ") || line.starts_with("### ") {
            current_heading = line.trim().to_string();
            i += 1;
            continue;
        }

        if is_admonition_start(line) {
            let start = i;
            i += 1;
            while i < lines.len() {
                let nxt = &lines[i];
                let is_continuation = nxt.starts_with("    ")
                    || nxt.starts_with('\t')
                    || (nxt.trim().is_empty()
                        && i + 1 < lines.len()
                        && (lines[i + 1].starts_with("    ") || lines[i + 1].starts_with('\t')));
                if is_continuation {
                    i += 1;
                } else {
                    break;
                }
            }
            let block = lines[start..i].join("\n").trim_end().to_string();
            blocks.push((current_heading.clone(), block));
            continue;
        }

        if is_mermaid_start(line) {
            let start = i;
            i += 1;
            while i < lines.len() && !lines[i].trim_start().starts_with("```") {
                i += 1;
            }
            if i < lines.len() {
                i += 1;
            }
            let block = lines[start..i].join("\n").trim_end().to_string();
            blocks.push((current_heading.clone(), block));
            continue;
        }

        i += 1;
    }

    blocks
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn preserve_manual_blocks(lines: Vec<String>, blocks: Vec<(String, String)>) -> Vec<String> {
    if blocks.is_empty() {
        return lines;
    }

    let current_norm = collapse_ws(&lines.join("\n"));
    let pending: Vec<(String, String)> = blocks
        .into_iter()
        .filter(|(_, block)| !current_norm.contains(&collapse_ws(block)))
        .collect();
    if pending.is_empty() {
        return lines;
    }

    let existing_headings: HashSet<String> = lines
        .iter()
        .filter(|l| l.starts_with("## ") || l.starts_with("### "))
        .map(|l| l.trim().to_string())
        .collect();

    let mut by_anchor: HashMap<String, Vec<String>> = HashMap::new();
    let mut fallback_before_source: Vec<String> = Vec::new();

    for (anchor, block) in pending {
        if existing_headings.contains(&anchor) && !anchor.to_ascii_lowercase().contains("source") {
            by_anchor.entry(anchor).or_default().push(block);
        } else {
            fallback_before_source.push(block);
        }
    }

    let mut out = Vec::new();
    let mut i = 0;
    let mut inserted_fallback = false;

    while i < lines.len() {
        let line = &lines[i];
        let trimmed = line.trim();

        if trimmed.starts_with("## ")
            && classify_h2(trimmed) == Some(StandardSection::Source)
            && !inserted_fallback
        {
            for blk in &fallback_before_source {
                if out.last().is_some_and(|l: &String| !l.trim().is_empty()) {
                    out.push(String::new());
                }
                out.extend(blk.lines().map(str::to_string));
                out.push(String::new());
            }
            inserted_fallback = true;
        }

        out.push(line.clone());
        if let Some(anchor_blocks) = by_anchor.get(trimmed) {
            i += 1;
            while i < lines.len() && !(lines[i].starts_with("## ") || lines[i].starts_with("### "))
            {
                out.push(lines[i].clone());
                i += 1;
            }
            for blk in anchor_blocks {
                if out.last().is_some_and(|l: &String| !l.trim().is_empty()) {
                    out.push(String::new());
                }
                out.extend(blk.lines().map(str::to_string));
                out.push(String::new());
            }
            continue;
        }

        i += 1;
    }

    if !fallback_before_source.is_empty() && !inserted_fallback {
        for blk in &fallback_before_source {
            if out.last().is_some_and(|l: &String| !l.trim().is_empty()) {
                out.push(String::new());
            }
            out.extend(blk.lines().map(str::to_string));
            out.push(String::new());
        }
    }

    out
}

fn has_content_tabs(lines: &[String]) -> bool {
    lines.iter().any(|l| l.trim_start().starts_with("=== \""))
}

fn merge_structured(existing: &str, compiled: &str) -> String {
    let (ex_fm, ex_body) = split_frontmatter(existing);
    let (co_fm, co_body) = split_frontmatter(compiled);
    let merged_fm = merge_frontmatter(ex_fm, co_fm);

    let ex_raw_lines: Vec<String> = ex_body.lines().map(str::to_string).collect();
    let co_raw_lines: Vec<String> = co_body.lines().map(str::to_string).collect();

    let (ex_lines, ex_defs) = extract_footer_defs(&ex_raw_lines);
    let (co_lines, co_defs) = extract_footer_defs(&co_raw_lines);

    let (ex_preamble, ex_sections) = split_h2_sections(&ex_lines);
    let (co_preamble, co_sections) = split_h2_sections(&co_lines);

    // 1. Preserve H1 emoji and image embed from existing preamble if missing in compiled
    let mut ex_h1_emoji: Option<String> = None;
    let mut ex_image_line: Option<String> = None;
    let mut ex_image_ref_keys: HashSet<String> = HashSet::new();
    for l in &ex_preamble {
        if let Some((Some(emoji), _)) = parse_h1(l) {
            ex_h1_emoji = Some(emoji.to_string());
        } else if is_image_embed(l) {
            ex_image_line = Some(l.trim().to_string());
            if let Some(ref_key) = extract_image_ref_key(l) {
                ex_image_ref_keys.insert(ref_key.to_ascii_lowercase());
            }
        }
    }
    let co_has_image = co_preamble.iter().any(|l| is_image_embed(l));

    let mut new_preamble = Vec::new();
    for l in &co_preamble {
        if let Some((co_emoji, title)) = parse_h1(l) {
            if co_emoji.is_none()
                && let Some(ref emoji) = ex_h1_emoji
            {
                new_preamble.push(format!("# {emoji} {title}"));
            } else {
                new_preamble.push(l.clone());
            }
            if !co_has_image && let Some(ref img) = ex_image_line {
                new_preamble.push(String::new());
                new_preamble.push(img.clone());
            }
            continue;
        }
        new_preamble.push(l.clone());
    }

    // 2. Partition existing sections into Source, manual tabs, and custom sections
    let co_headings_lower: HashSet<String> = co_sections
        .iter()
        .map(|(h, _)| h.to_ascii_lowercase())
        .collect();
    let mut ex_source: Option<(String, Vec<String>)> = None;
    let mut ex_tab_sections: HashMap<StandardSection, (String, Vec<String>)> = HashMap::new();
    let mut custom_before_source: Vec<(String, Vec<String>)> = Vec::new();
    let mut custom_after_source: Vec<(String, Vec<String>)> = Vec::new();
    let mut seen_source = false;

    for (heading, sec_lines) in &ex_sections {
        match classify_h2(heading) {
            Some(StandardSection::Source) => {
                seen_source = true;
                ex_source = Some((heading.clone(), sec_lines.clone()));
            }
            Some(kind @ (StandardSection::Ingredients | StandardSection::Instructions)) => {
                if has_content_tabs(sec_lines) {
                    ex_tab_sections.insert(kind, (heading.clone(), sec_lines.clone()));
                }
            }
            Some(_) => {}
            None => {
                if !co_headings_lower.contains(&heading.to_ascii_lowercase()) {
                    if seen_source {
                        custom_after_source.push((heading.clone(), sec_lines.clone()));
                    } else {
                        custom_before_source.push((heading.clone(), sec_lines.clone()));
                    }
                }
            }
        }
    }

    // 3. Rebuild sections from compiled, preserving manual tabs, source links, and custom sections
    let mut merged_sections: Vec<(String, Vec<String>)> = Vec::new();
    let has_compiled_source = co_sections
        .iter()
        .any(|(h, _)| classify_h2(h) == Some(StandardSection::Source));

    for (heading, sec_lines) in co_sections {
        let kind = classify_h2(&heading);
        if let Some(k @ (StandardSection::Ingredients | StandardSection::Instructions)) = kind
            && let Some(existing_tabbed) = ex_tab_sections.get(&k)
            && !has_content_tabs(&sec_lines)
        {
            log::info!(
                "Preserved manual content tabs in '{}' (pass --force to overwrite)",
                heading
            );
            merged_sections.push(existing_tabbed.clone());
            continue;
        }

        if kind == Some(StandardSection::Source) {
            merged_sections.append(&mut custom_before_source);
            if let Some((_, ref ex_src_lines)) = ex_source {
                merged_sections.push((
                    heading,
                    merge_source_lines(ex_src_lines, &sec_lines, &ex_defs),
                ));
            } else {
                merged_sections.push((heading, sec_lines));
            }
            merged_sections.append(&mut custom_after_source);
            continue;
        }

        merged_sections.push((heading, sec_lines));
    }

    if !has_compiled_source {
        merged_sections.extend(custom_before_source);
        if let Some((ex_h, ex_src_lines)) = ex_source {
            merged_sections.push((ex_h, ex_src_lines));
        }
        merged_sections.extend(custom_after_source);
    }

    let mut body_lines = new_preamble;
    for (heading, sec_lines) in merged_sections {
        if body_lines.last().is_some_and(|l| !l.trim().is_empty()) {
            body_lines.push(String::new());
        }
        body_lines.push(heading);
        body_lines.extend(sec_lines);
    }

    // 4. Preserve manual admonitions and Mermaid diagrams from existing body
    let manual_blocks = extract_manual_blocks(&ex_lines);
    body_lines = preserve_manual_blocks(body_lines, manual_blocks);

    // 5. Merge footer reference / footnote definitions without duplicates or orphaned image refs
    let mut merged_defs = co_defs;
    let mut seen_def_keys: HashSet<String> = merged_defs
        .iter()
        .map(|(k, _)| k.to_ascii_lowercase())
        .collect();
    for (k, block) in ex_defs {
        let k_lower = k.to_ascii_lowercase();
        if seen_def_keys.contains(&k_lower) {
            continue;
        }
        if co_has_image
            && ex_image_ref_keys.contains(&k_lower)
            && !body_references_key(&body_lines, &k)
        {
            continue;
        }
        seen_def_keys.insert(k_lower);
        merged_defs.push((k, block));
    }

    // Assemble final document and normalize consecutive blank lines cleanly
    let mut out = String::new();
    if let Some(ref fm) = merged_fm {
        out.push_str(&format_frontmatter_yaml(fm));
    }
    out.push_str(&body_lines.join("\n"));
    if !merged_defs.is_empty() {
        if !out.ends_with("\n\n") {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
        }
        for (idx, (_, block)) in merged_defs.iter().enumerate() {
            if idx > 0 {
                out.push('\n');
            }
            out.push_str(block);
        }
    }

    let formatted = crate::document::normalize_markdown_spacing(&out);
    let trimmed = formatted.trim_end_matches('\n');
    format!("{trimmed}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_marker_region_preserves_surrounding_and_is_idempotent() {
        let existing = "\
---
comments: true
custom_note: keep me
---
# Custom Header

Intro paragraph written manually.

<!-- cook-docs:begin -->
OLD GENERATED CONTENT
<!-- cook-docs:end -->

## Personal Notes

Do not touch this section!
";
        let compiled = "## :salt: Ingredients\n\n- 2 cups flour\n";

        let merged1 = merge_markdown(existing, compiled);
        assert!(merged1.contains("custom_note: keep me"));
        assert!(merged1.contains("Intro paragraph written manually."));
        assert!(merged1.contains("- 2 cups flour"));
        assert!(!merged1.contains("OLD GENERATED CONTENT"));
        assert!(merged1.contains("## Personal Notes\n\nDo not touch this section!"));

        // Idempotent on second merge
        let merged2 = merge_markdown(&merged1, compiled);
        assert_eq!(merged1, merged2);

        // Compiled containing markers itself does not duplicate markers
        let compiled_with_markers =
            format!("preamble\n{BEGIN_MARKER}\n- 3 cups flour\n{END_MARKER}\ntrailer\n");
        let merged3 = merge_markdown(&merged1, &compiled_with_markers);
        assert_eq!(merged3.matches(BEGIN_MARKER).count(), 1);
        assert_eq!(merged3.matches(END_MARKER).count(), 1);
        assert!(merged3.contains("- 3 cups flour"));
    }

    #[test]
    fn test_merge_structured_preserves_frontmatter_sections_admonitions_mermaid_tabs_and_defs() {
        let existing = "\
---
comments: true
hero: assets/images/custom-bread.webp
rating: 5
tags:
  - bread
  - favorite
---

# :bread: Sourdough Loaf

![Sourdough Loaf](../assets/images/custom-bread.webp){ loading=lazy }

| :fork_and_knife_with_plate: Serves | :timer_clock: Total Time |
|:----------------------------------:|:-----------------------: |
| 8 | 4 hours |

## :salt: Ingredients

=== \"1x Batch\"

    - 500 g flour

=== \"1/2 Batch\"

    - 250 g flour

## :cooking: Cookware

- 1 old bowl

## :pencil: Instructions

### Step 1

Old step text.

!!! tip \"Pro Tip\"
    Use cold water in summer.

## :calendar: Schedule

```mermaid
flowchart LR
  Mix --> Bulk --> Bake
```

## :link: Source

- [Artisan Bread Guide][1]

## :memo: Tasting Notes

Great crust with steam[^1].

[1]: https://example.com/sourdough
[^1]: Tested at 450F.
";

        let compiled = "\
---
comments: true
tags:
  - bread
  - sourdough
---

# Sourdough Loaf

| :fork_and_knife_with_plate: Serves | :timer_clock: Total Time |
|:----------------------------------:|:-----------------------: |
| 8 | 4 hours |

## :salt: Ingredients

- 500 g bread flour

## :cooking: Cookware

- 1 dutch oven

## :pencil: Instructions

### Step 1

Mix bread flour and water thoroughly.

## :link: Source

- <https://example.com/sourdough>
- <https://example.com/extra-ref>
";

        let merged = merge_markdown(existing, compiled);

        // Frontmatter: unioned tags, preserved manual hero & rating
        assert!(
            merged.contains("hero: assets/images/custom-bread.webp"),
            "merged:\n{merged}"
        );
        assert!(merged.contains("rating: 5"), "merged:\n{merged}");
        assert!(merged.contains("- bread"), "merged:\n{merged}");
        assert!(merged.contains("- favorite"), "merged:\n{merged}");
        assert!(merged.contains("- sourdough"), "merged:\n{merged}");

        // H1 emoji & image embed preserved
        assert!(
            merged.contains("# :bread: Sourdough Loaf"),
            "merged:\n{merged}"
        );
        assert!(
            merged
                .contains("![Sourdough Loaf](../assets/images/custom-bread.webp){ loading=lazy }"),
            "merged:\n{merged}"
        );

        // Manual tabs preserved in Ingredients
        assert!(merged.contains("=== \"1/2 Batch\""), "merged:\n{merged}");

        // Standard non-tabbed sections updated from compiled
        assert!(merged.contains("- 1 dutch oven"), "merged:\n{merged}");
        assert!(!merged.contains("- 1 old bowl"), "merged:\n{merged}");
        assert!(
            merged.contains("Mix bread flour and water thoroughly."),
            "merged:\n{merged}"
        );

        // Admonition preserved under ### Step 1
        assert!(
            merged.contains("!!! tip \"Pro Tip\"\n    Use cold water in summer."),
            "merged:\n{merged}"
        );

        // Custom sections & Mermaid diagram preserved in relative order around Source
        assert!(
            merged.contains("## :calendar: Schedule"),
            "merged:\n{merged}"
        );
        assert!(
            merged.contains("```mermaid\nflowchart LR\n  Mix --> Bulk --> Bake\n```"),
            "merged:\n{merged}"
        );
        assert!(
            merged.contains("## :memo: Tasting Notes"),
            "merged:\n{merged}"
        );
        let sched_pos = merged.find("## :calendar: Schedule").unwrap();
        let src_pos = merged.find("## :link: Source").unwrap();
        let notes_pos = merged.find("## :memo: Tasting Notes").unwrap();
        assert!(sched_pos < src_pos && src_pos < notes_pos);

        // Source merged without duplicating existing URL, appending new URL
        assert!(
            merged.contains("- [Artisan Bread Guide][1]"),
            "merged:\n{merged}"
        );
        assert!(
            merged.contains("- <https://example.com/extra-ref>"),
            "merged:\n{merged}"
        );
        assert_eq!(merged.matches("https://example.com/sourdough").count(), 1);

        // Reference link & footnote preserved
        assert!(
            merged.contains("[1]: https://example.com/sourdough"),
            "merged:\n{merged}"
        );
        assert!(
            merged.contains("[^1]: Tested at 450F."),
            "merged:\n{merged}"
        );

        // Strictly idempotent on repeated merge
        let merged_again = merge_markdown(&merged, compiled);
        assert_eq!(merged, merged_again);
    }

    #[test]
    fn test_merge_without_frontmatter_does_not_inject_frontmatter() {
        let existing = "\
# Soup

## :salt: Ingredients

- 1 cup water

## Notes

Serve hot.
";
        let compiled = "\
# Soup

## :salt: Ingredients

- 2 cups water
";
        let merged = merge_markdown(existing, compiled);
        assert!(!merged.starts_with("---"));
        assert!(merged.contains("- 2 cups water"));
        assert!(merged.contains("## Notes\n\nServe hot."));
        assert_eq!(merged, merge_markdown(&merged, compiled));
    }

    #[test]
    fn test_merge_drops_orphaned_image_ref_def_and_preserves_referenced() {
        // 1. Existing [1] ref image + compiled inline image -> drops orphaned [1]: def
        let existing_ref = "\
# :bread: Bagels

![Bagels][1]{ loading=lazy }

## :salt: Ingredients

- 500 g flour

## :link: Source

- [King Arthur][2]

[1]: <../assets/images/bagels.webp>
[2]: https://example.com/bagels
[^1]: Boiled in barley malt water.
";
        let compiled_inline = "\
# :bread: Bagels

![Bagels](../assets/images/bagels.webp){ loading=lazy }

## :salt: Ingredients

- 500 g bread flour

## :link: Source

- <https://example.com/bagels>
";
        let merged_inline = merge_markdown(existing_ref, compiled_inline);
        assert!(
            merged_inline.contains("![Bagels](../assets/images/bagels.webp){ loading=lazy }"),
            "merged:\n{merged_inline}"
        );
        assert!(
            !merged_inline.contains("[1]: <../assets/images/bagels.webp>"),
            "orphan [1]: should be dropped:\n{merged_inline}"
        );
        assert!(
            merged_inline.contains("[2]: https://example.com/bagels"),
            "unrelated ref [2]: should be preserved:\n{merged_inline}"
        );
        assert!(
            merged_inline.contains("[^1]: Boiled in barley malt water."),
            "footnote should be preserved:\n{merged_inline}"
        );
        assert_eq!(
            merged_inline,
            merge_markdown(&merged_inline, compiled_inline)
        );

        // 2. Existing [1] ref image + compiled [1] ref image -> updates [1]: cleanly
        let existing_old_ref = "\
# Bagels

![Bagels][1]{ loading=lazy }

## :salt: Ingredients

- 500 g flour

[1]: <../assets/images/old-bagels.webp>
";
        let compiled_new_ref = "\
# Bagels

![Bagels][1]{ loading=lazy }

## :salt: Ingredients

- 500 g bread flour

[1]: <../assets/images/new-bagels.webp>
";
        let merged_ref = merge_markdown(existing_old_ref, compiled_new_ref);
        assert!(
            merged_ref.contains("[1]: <../assets/images/new-bagels.webp>"),
            "merged:\n{merged_ref}"
        );
        assert!(
            !merged_ref.contains("old-bagels.webp"),
            "old ref should be replaced:\n{merged_ref}"
        );
        assert_eq!(merged_ref.matches("[1]:").count(), 1);

        // 3. Existing [1] ref image + [1] also referenced in body text -> preserves [1]:
        let existing_shared_ref = "\
# Bagels

![Bagels][1]{ loading=lazy }

## :salt: Ingredients

- 500 g flour

## :memo: Notes

Compare against [the original photo][1] for crust color.

[1]: <../assets/images/bagels.webp>
";
        let merged_shared = merge_markdown(existing_shared_ref, compiled_inline);
        assert!(
            merged_shared.contains("[1]: <../assets/images/bagels.webp>"),
            "referenced [1]: in body must be preserved:\n{merged_shared}"
        );

        // 4. Existing [1] ref image + compiled without image -> preserves both embed and [1]:
        let compiled_no_img = "\
# Bagels

## :salt: Ingredients

- 500 g bread flour
";
        let merged_no_img = merge_markdown(existing_old_ref, compiled_no_img);
        assert!(
            merged_no_img.contains("![Bagels][1]{ loading=lazy }"),
            "merged:\n{merged_no_img}"
        );
        assert!(
            merged_no_img.contains("[1]: <../assets/images/old-bagels.webp>"),
            "merged:\n{merged_no_img}"
        );
    }
}

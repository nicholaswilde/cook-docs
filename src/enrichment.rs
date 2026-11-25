use crate::types::Config;
use cooklang::convert::{ConvertTo, ConvertUnit, ConvertValue, Converter, PhysicalQuantity, Unit};
use figment::{
    Figment,
    providers::{Format, Json, Toml, Yaml},
};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

const PREP_WORDS: &[&str] = &[
    "fresh",
    "freshly",
    "chopped",
    "minced",
    "diced",
    "sliced",
    "slivered",
    "grated",
    "shredded",
    "crushed",
    "peeled",
    "pitted",
    "seeded",
    "squeezed",
    "melted",
    "softened",
    "sifted",
    "unsifted",
    "packed",
    "lightly",
    "finely",
    "coarsely",
    "roughly",
    "thinly",
    "cold",
    "warm",
    "hot",
    "room",
    "temperature",
    "large",
    "medium",
    "small",
    "raw",
    "organic",
    "pure",
    "extra",
    "unsalted",
    "salted",
    "optional",
];

fn normalize_text(raw: &str) -> String {
    let cleaned = raw.replace('\u{00ad}', "").replace('\u{2019}', "'");
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let lower = collapsed.to_ascii_lowercase();
    lower
        .strip_prefix("some ")
        .unwrap_or(&lower)
        .trim()
        .to_string()
}

fn singularize_phrase(phrase: &str) -> Option<String> {
    let mut parts: Vec<String> = phrase.split_whitespace().map(|s| s.to_string()).collect();
    let last = parts.last_mut()?;
    let singular = if last.ends_with("ies") && last.len() > 4 {
        format!("{}y", &last[..last.len() - 3])
    } else if last.ends_with("es")
        && last.len() > 3
        && !last.ends_with("ses")
        && !last.ends_with("xes")
        && !last.ends_with("zes")
    {
        if last.ends_with("oes") || last.ends_with("ches") || last.ends_with("shes") {
            last[..last.len() - 2].to_string()
        } else {
            last[..last.len() - 1].to_string()
        }
    } else if last.ends_with('s') && last.len() > 2 && !last.ends_with("ss") {
        last[..last.len() - 1].to_string()
    } else {
        return None;
    };
    *last = singular;
    Some(parts.join(" "))
}

fn strip_prep_words(phrase: &str) -> String {
    phrase
        .split_whitespace()
        .filter(|w| {
            let trimmed = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '\'');
            !trimmed.is_empty() && !PREP_WORDS.contains(&trimmed)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn pluralize_candidates(seed: &str) -> Vec<String> {
    let mut plurals = vec![format!("{seed}s")];
    if seed.ends_with('o')
        || seed.ends_with("ch")
        || seed.ends_with("sh")
        || seed.ends_with('x')
        || seed.ends_with('z')
    {
        plurals.push(format!("{seed}es"));
    }
    if seed.ends_with('y') && seed.len() > 2 {
        let prev = seed.as_bytes()[seed.len() - 2];
        if !matches!(prev, b'a' | b'e' | b'i' | b'o' | b'u') {
            plurals.push(format!("{}ies", &seed[..seed.len() - 1]));
        }
    }
    plurals
}

fn lookup_candidates(name: &str) -> Vec<String> {
    let norm = normalize_text(name);
    if norm.is_empty() {
        return Vec::new();
    }

    let base = norm
        .split([',', '('])
        .next()
        .map(str::trim)
        .unwrap_or(&norm)
        .to_string();
    let stripped = strip_prep_words(&base);

    let mut out = Vec::new();
    let mut push = |cand: String| {
        if !cand.is_empty() && !out.contains(&cand) {
            out.push(cand);
        }
    };

    for seed in [norm, base, stripped] {
        if seed.is_empty() {
            continue;
        }
        if let Some(sing) = singularize_phrase(&seed) {
            push(seed.clone());
            push(sing);
        } else {
            push(seed.clone());
        }
        if let Some(s_trimmed) = seed.strip_suffix('s')
            && s_trimmed.len() >= 2
        {
            push(s_trimmed.to_string());
        }
        for pl in pluralize_candidates(&seed) {
            push(pl);
        }
    }

    out
}

fn load_structured_file(path: &Path) -> Result<JsonValue, String> {
    if !path.exists() {
        return Err(format!("File '{}' not found", path.display()));
    }
    if !path.is_file() {
        return Err(format!("Path '{}' is not a regular file", path.display()));
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());

    let figment = match ext.as_deref() {
        Some("toml") => Figment::new().merge(Toml::file(path)),
        Some("yaml") | Some("yml") => Figment::new().merge(Yaml::file(path)),
        Some("json") => Figment::new().merge(Json::file(path)),
        _ => {
            return Err(format!(
                "Unsupported file format '{}': expected .toml, .yaml, .yml, or .json",
                path.display()
            ));
        }
    };

    figment
        .extract::<JsonValue>()
        .map_err(|e| format!("Failed to parse '{}': {e}", path.display()))
}

fn json_scalar_to_string(v: &JsonValue) -> Option<String> {
    match v {
        JsonValue::String(s) => Some(s.clone()),
        JsonValue::Number(n) => Some(n.to_string()),
        JsonValue::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn format_emoji_token(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with(':') && trimmed.ends_with(':') && trimmed.len() >= 3 {
        return trimmed.to_string();
    }
    if trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+'))
    {
        format!(":{trimmed}:")
    } else {
        trimmed.to_string()
    }
}

/// Return `true` when `title` already begins with a `:shortcode:` or Unicode emoji glyph.
pub fn has_leading_emoji(title: &str) -> bool {
    let trimmed = title.trim_start();
    if let Some(after_colon) = trimmed.strip_prefix(':')
        && let Some(end_colon) = after_colon.find(':')
        && end_colon > 0
    {
        let valid_shortcode = after_colon[..end_colon]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+'));
        if valid_shortcode {
            return true;
        }
    }
    trimmed.chars().next().is_some_and(|c| {
        matches!(
            c as u32,
            0x2300..=0x23FF | 0x2600..=0x27BF | 0x2B00..=0x2BFF | 0x1F000..=0x1FAFF
        )
    })
}

fn is_explicit_emoji_token(raw: &str) -> bool {
    let trimmed = raw.trim();
    (trimmed.starts_with(':') && trimmed.ends_with(':') && trimmed.len() >= 3)
        || (!trimmed.is_empty() && !trimmed.is_ascii())
}

fn normalize_category_key(raw: &str) -> String {
    raw.trim()
        .replace('\\', "/")
        .split('/')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join("/")
}

fn title_words(title: &str) -> Vec<String> {
    normalize_text(title)
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .map(|w| w.trim_matches('\''))
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect()
}

/// Ingredient, cookware, title, and category emoji mapping table (`#11`, `#79`).
#[derive(Debug, Clone, Default)]
pub struct EmojiTable {
    ingredients: HashMap<String, String>,
    cookware: HashMap<String, String>,
    titles: HashMap<String, String>,
    categories: HashMap<String, String>,
}

impl EmojiTable {
    /// Load emoji mappings from `config.emoji_file` and/or inline `config.emoji`.
    pub fn load(config: &Config) -> Result<Self, String> {
        let mut table = Self::default();
        if !config.emoji_file.trim().is_empty() {
            let val = load_structured_file(Path::new(config.emoji_file.trim()))?;
            table.merge_value(&val)?;
        }
        if let Some(inline) = &config.emoji {
            table.merge_value(inline)?;
        }
        Ok(table)
    }

    /// Parse emoji mappings directly from a structured JSON/YAML/TOML value.
    pub fn from_value(val: &JsonValue) -> Result<Self, String> {
        let mut table = Self::default();
        table.merge_value(val)?;
        Ok(table)
    }

    fn merge_value(&mut self, val: &JsonValue) -> Result<(), String> {
        let Some(root) = val.as_object() else {
            return Err("Emoji configuration must be a mapping/object".to_string());
        };
        let target = root
            .get("emoji")
            .and_then(JsonValue::as_object)
            .unwrap_or(root);

        if let Some(ing) = target.get("ingredients") {
            Self::parse_category(ing, &mut self.ingredients)?;
        }
        if let Some(cw) = target.get("cookware") {
            Self::parse_category(cw, &mut self.cookware)?;
        }
        if let Some(titles) = target.get("titles") {
            Self::parse_titles(titles, &mut self.titles)?;
        }
        if let Some(categories) = target.get("categories") {
            Self::parse_categories_map(categories, &mut self.categories)?;
        }
        Ok(())
    }

    fn parse_category(val: &JsonValue, out: &mut HashMap<String, String>) -> Result<(), String> {
        Self::parse_group_list_or_map(val, out, normalize_text)
    }

    fn parse_group_list_or_map(
        val: &JsonValue,
        out: &mut HashMap<String, String>,
        normalize: fn(&str) -> String,
    ) -> Result<(), String> {
        match val {
            JsonValue::Array(groups) => {
                for group in groups {
                    let Some(map) = group.as_object() else {
                        return Err(
                            "Each emoji list entry must map an emoji name to a list of items"
                                .to_string(),
                        );
                    };
                    for (emoji_key, items_val) in map {
                        Self::insert_items_with(emoji_key, items_val, out, normalize);
                    }
                }
                Ok(())
            }
            JsonValue::Object(map) => {
                for (emoji_key, items_val) in map {
                    Self::insert_items_with(emoji_key, items_val, out, normalize);
                }
                Ok(())
            }
            JsonValue::Null => Ok(()),
            _ => Err("Emoji category must be a list or mapping".to_string()),
        }
    }

    fn parse_titles(val: &JsonValue, out: &mut HashMap<String, String>) -> Result<(), String> {
        match val {
            JsonValue::Array(_) | JsonValue::Null => Self::parse_category(val, out),
            JsonValue::Object(map) => {
                for (k, v) in map {
                    if !v.is_array()
                        && let Some(s) = json_scalar_to_string(v)
                        && !is_explicit_emoji_token(k)
                        && is_explicit_emoji_token(&s)
                    {
                        Self::insert_pair(k, &s, out, normalize_text);
                    } else {
                        Self::insert_items_with(k, v, out, normalize_text);
                    }
                }
                Ok(())
            }
            _ => Err("Emoji titles must be a list or mapping".to_string()),
        }
    }

    fn parse_categories_map(
        val: &JsonValue,
        out: &mut HashMap<String, String>,
    ) -> Result<(), String> {
        match val {
            JsonValue::Array(_) | JsonValue::Null => {
                Self::parse_group_list_or_map(val, out, normalize_category_key)
            }
            JsonValue::Object(map) => {
                for (k, v) in map {
                    if v.is_array() || is_explicit_emoji_token(k) {
                        Self::insert_items_with(k, v, out, normalize_category_key);
                    } else if let Some(s) = json_scalar_to_string(v) {
                        Self::insert_pair(k, &s, out, normalize_category_key);
                    }
                }
                Ok(())
            }
            _ => Err("Emoji categories must be a list or mapping".to_string()),
        }
    }

    fn insert_pair(
        raw_key: &str,
        raw_emoji: &str,
        out: &mut HashMap<String, String>,
        normalize: fn(&str) -> String,
    ) {
        let norm = normalize(raw_key);
        let formatted = format_emoji_token(raw_emoji);
        if !norm.is_empty() && !formatted.is_empty() {
            out.insert(norm, formatted);
        }
    }

    fn insert_items_with(
        emoji_key: &str,
        items_val: &JsonValue,
        out: &mut HashMap<String, String>,
        normalize: fn(&str) -> String,
    ) {
        let formatted = format_emoji_token(emoji_key);
        if formatted.is_empty() {
            return;
        }
        let items: &[JsonValue] = match items_val {
            JsonValue::Array(arr) => arr.as_slice(),
            other => std::slice::from_ref(other),
        };
        for item in items {
            if let Some(s) = json_scalar_to_string(item) {
                let norm = normalize(&s);
                if !norm.is_empty() {
                    out.insert(norm, formatted.clone());
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.ingredients.is_empty()
            && self.cookware.is_empty()
            && self.titles.is_empty()
            && self.categories.is_empty()
    }

    pub fn counts(&self) -> (usize, usize) {
        (self.ingredients.len(), self.cookware.len())
    }

    pub fn lookup_ingredient(&self, name: &str) -> Option<&str> {
        Self::lookup_in(&self.ingredients, name)
    }

    pub fn lookup_cookware(&self, name: &str) -> Option<&str> {
        Self::lookup_in(&self.cookware, name)
    }

    pub fn lookup_category(&self, categories: &[String]) -> Option<&str> {
        if self.categories.is_empty() || categories.is_empty() {
            return None;
        }
        let segs: Vec<String> = categories
            .iter()
            .flat_map(|c| {
                c.replace('\\', "/")
                    .split('/')
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        if segs.is_empty() {
            return None;
        }
        for start in 0..segs.len() {
            let key = segs[start..].join("/");
            if let Some(emoji) = self.categories.get(&key) {
                return Some(emoji.as_str());
            }
        }
        for end in (1..segs.len()).rev() {
            let key = segs[..end].join("/");
            if let Some(emoji) = self.categories.get(&key) {
                return Some(emoji.as_str());
            }
        }
        for seg in segs.iter().rev() {
            if let Some(emoji) = Self::lookup_in(&self.categories, seg) {
                return Some(emoji);
            }
        }
        None
    }

    pub fn lookup_title(&self, title: &str, categories: &[String]) -> Option<&str> {
        if has_leading_emoji(title) {
            return None;
        }
        if !title.trim().is_empty() {
            if let Some(emoji) = Self::lookup_title_in(&self.titles, title) {
                return Some(emoji);
            }
            if let Some(emoji) = Self::lookup_title_in(&self.ingredients, title) {
                return Some(emoji);
            }
        }
        self.lookup_category(categories)
    }

    pub fn lookup_any(&self, name: &str) -> Option<&str> {
        self.lookup_ingredient(name)
            .or_else(|| self.lookup_cookware(name))
    }

    fn lookup_in<'a>(map: &'a HashMap<String, String>, name: &str) -> Option<&'a str> {
        if map.is_empty() {
            return None;
        }
        for cand in lookup_candidates(name) {
            if let Some(emoji) = map.get(&cand) {
                return Some(emoji.as_str());
            }
        }
        None
    }

    fn lookup_title_in<'a>(map: &'a HashMap<String, String>, title: &str) -> Option<&'a str> {
        if map.is_empty() {
            return None;
        }
        if let Some(emoji) = Self::lookup_in(map, title) {
            return Some(emoji);
        }
        for raw_tok in normalize_text(title).split_whitespace().rev() {
            let trimmed = raw_tok.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
            if trimmed.contains('-')
                && let Some(emoji) = Self::lookup_in(map, trimmed)
            {
                return Some(emoji);
            }
        }
        let words = title_words(title);
        if words.is_empty() {
            return None;
        }
        let max_len = words.len().min(4);
        for len in (1..=max_len).rev() {
            for start in (0..=words.len() - len).rev() {
                let phrase = words[start..start + len].join(" ");
                if let Some(emoji) = Self::lookup_in(map, &phrase) {
                    return Some(emoji);
                }
            }
        }
        None
    }
}

fn find_unit_case_insensitive(converter: &Converter, raw: &str) -> Option<Arc<Unit>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(u) = converter.find_unit(trimmed) {
        return Some(u);
    }
    let lower = trimmed.to_ascii_lowercase();
    if let Some(u) = converter.find_unit(&lower) {
        return Some(u);
    }
    let no_dot = lower.trim_end_matches('.');
    if no_dot != lower
        && let Some(u) = converter.find_unit(no_dot)
    {
        return Some(u);
    }
    if matches!(no_dot, "fluid oz" | "fl.oz") {
        return converter.find_unit("fl oz");
    }
    None
}

fn volume_to_ml(converter: &Converter, qty: f64, unit_str: &str) -> Option<(f64, Arc<Unit>)> {
    if !qty.is_finite() || qty <= 0.0 {
        return None;
    }
    let unit = find_unit_case_insensitive(converter, unit_str)?;
    if unit.physical_quantity != PhysicalQuantity::Volume {
        return None;
    }
    let (ConvertValue::Number(ml), _) = converter
        .convert(
            ConvertValue::Number(qty),
            ConvertUnit::Unit(&unit),
            ConvertTo::Unit(ConvertUnit::Key("ml")),
        )
        .ok()?
    else {
        return None;
    };
    (ml.is_finite() && ml > 0.0).then_some((ml, unit))
}

fn parse_volume_expr(expr: &str) -> Option<(f64, String)> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return None;
    }
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    let parse_part = |s: &str| -> Option<f64> {
        if let Some((n, d)) = s.split_once('/') {
            let num: f64 = n.parse().ok()?;
            let den: f64 = d.parse().ok()?;
            (den != 0.0).then_some(num / den)
        } else {
            s.parse::<f64>().ok()
        }
    };

    if let Some(first_val) = parse_part(tokens[0]) {
        if tokens.len() >= 3
            && tokens[1].contains('/')
            && let Some(frac) = parse_part(tokens[1])
        {
            return Some((first_val + frac, tokens[2..].join(" ")));
        }
        let unit = if tokens.len() >= 2 {
            tokens[1..].join(" ")
        } else {
            "cup".to_string()
        };
        Some((first_val, unit))
    } else {
        Some((1.0, trimmed.to_string()))
    }
}

/// Ingredient density lookup table backed by `cooklang::Converter` (`#69`).
#[derive(Debug, Clone)]
pub struct WeightsTable {
    grams_per_ml: HashMap<String, f64>,
    exclude: HashSet<String>,
    exclude_small_units: HashSet<String>,
    exclude_small_units_except: HashSet<String>,
    converter: Converter,
}

impl Default for WeightsTable {
    fn default() -> Self {
        Self {
            grams_per_ml: HashMap::new(),
            exclude: HashSet::new(),
            exclude_small_units: HashSet::new(),
            exclude_small_units_except: HashSet::new(),
            converter: Converter::bundled(),
        }
    }
}

impl WeightsTable {
    /// Load density mappings from a YAML, TOML, or JSON file.
    pub fn load(path: &str) -> Result<Self, String> {
        if path.trim().is_empty() {
            return Ok(Self::default());
        }
        let val = load_structured_file(Path::new(path.trim()))?;
        Self::from_value(&val)
    }

    fn insert_normalized_strings(val: Option<&JsonValue>, out: &mut HashSet<String>) {
        if let Some(arr) = val.and_then(JsonValue::as_array) {
            for item in arr {
                if let Some(s) = json_scalar_to_string(item) {
                    let norm = normalize_text(&s);
                    if !norm.is_empty() {
                        out.insert(norm);
                    }
                }
            }
        }
    }

    /// Parse density mappings from a structured JSON/YAML/TOML value.
    pub fn from_value(val: &JsonValue) -> Result<Self, String> {
        let Some(root) = val.as_object() else {
            return Err("Weights file must be a mapping/object".to_string());
        };
        let mut table = Self::default();
        let ml_per_cup = volume_to_ml(&table.converter, 1.0, "cup")
            .map(|(ml, _)| ml)
            .unwrap_or(236.588_236);

        Self::insert_normalized_strings(root.get("exclude"), &mut table.exclude);
        Self::insert_normalized_strings(
            root.get("exclude_small_units")
                .or_else(|| root.get("exclude-small-units"))
                .or_else(|| root.get("excludeSmallUnits")),
            &mut table.exclude_small_units,
        );
        Self::insert_normalized_strings(
            root.get("exclude_small_units_except")
                .or_else(|| root.get("exclude-small-units-except"))
                .or_else(|| root.get("excludeSmallUnitsExcept")),
            &mut table.exclude_small_units_except,
        );

        let entries = root
            .get("weights")
            .and_then(JsonValue::as_object)
            .unwrap_or(root);

        for (raw_name, entry_val) in entries {
            if matches!(
                raw_name.as_str(),
                "exclude"
                    | "exclude_small_units"
                    | "exclude-small-units"
                    | "excludeSmallUnits"
                    | "exclude_small_units_except"
                    | "exclude-small-units-except"
                    | "excludeSmallUnitsExcept"
            ) && entry_val.is_array()
            {
                continue;
            }
            let (density, aliases) =
                Self::parse_entry(&table.converter, ml_per_cup, raw_name, entry_val)?;
            let norm = normalize_text(raw_name);
            if !norm.is_empty() {
                table.grams_per_ml.insert(norm, density);
            }
            for alias in aliases {
                let norm_alias = normalize_text(&alias);
                if !norm_alias.is_empty() {
                    table.grams_per_ml.insert(norm_alias, density);
                }
            }
        }

        Ok(table)
    }

    fn parse_entry(
        converter: &Converter,
        ml_per_cup: f64,
        name: &str,
        val: &JsonValue,
    ) -> Result<(f64, Vec<String>), String> {
        if let Some(g_per_cup) = val.as_f64() {
            if g_per_cup <= 0.0 || !g_per_cup.is_finite() {
                return Err(format!("Weight for '{name}' must be positive"));
            }
            return Ok((g_per_cup / ml_per_cup, Vec::new()));
        }

        let Some(obj) = val.as_object() else {
            return Err(format!(
                "Invalid weight entry for '{name}': expected a number (grams per cup) or object"
            ));
        };

        let aliases: Vec<String> = obj
            .get("aliases")
            .and_then(JsonValue::as_array)
            .map(|arr| arr.iter().filter_map(json_scalar_to_string).collect())
            .unwrap_or_default();

        if let Some(g_per_ml) = obj
            .get("grams_per_ml")
            .or_else(|| obj.get("grams-per-ml"))
            .or_else(|| obj.get("gramsPerMl"))
            .or_else(|| obj.get("g_per_ml"))
            .or_else(|| obj.get("density"))
            .and_then(JsonValue::as_f64)
        {
            if g_per_ml <= 0.0 || !g_per_ml.is_finite() {
                return Err(format!("Density for '{name}' must be positive"));
            }
            return Ok((g_per_ml, aliases));
        }

        if let Some(g_per_cup) = obj
            .get("grams_per_cup")
            .or_else(|| obj.get("grams-per-cup"))
            .or_else(|| obj.get("gramsPerCup"))
            .or_else(|| obj.get("g_per_cup"))
            .and_then(JsonValue::as_f64)
        {
            if g_per_cup <= 0.0 || !g_per_cup.is_finite() {
                return Err(format!("grams_per_cup for '{name}' must be positive"));
            }
            return Ok((g_per_cup / ml_per_cup, aliases));
        }

        if let Some(grams) = obj.get("grams").and_then(JsonValue::as_f64) {
            if grams <= 0.0 || !grams.is_finite() {
                return Err(format!("grams for '{name}' must be positive"));
            }
            let (ref_qty, ref_unit) =
                if let Some(vol) = obj.get("volume").and_then(JsonValue::as_str) {
                    parse_volume_expr(vol)
                        .ok_or_else(|| format!("Invalid volume expression '{vol}' for '{name}'"))?
                } else if let Some(unit) = obj.get("unit").and_then(JsonValue::as_str) {
                    let qty = obj
                        .get("quantity")
                        .and_then(JsonValue::as_f64)
                        .unwrap_or(1.0);
                    (qty, unit.to_string())
                } else {
                    (1.0, "cup".to_string())
                };
            let (ml, _) = volume_to_ml(converter, ref_qty, &ref_unit)
                .ok_or_else(|| format!("Unsupported volume unit '{ref_unit}' for '{name}'"))?;
            return Ok((grams / ml, aliases));
        }

        Err(format!(
            "Weight entry for '{name}' must define grams_per_cup, grams_per_ml, density, or grams"
        ))
    }

    pub fn is_empty(&self) -> bool {
        self.grams_per_ml.is_empty()
    }

    pub fn len(&self) -> usize {
        self.grams_per_ml.len()
    }

    fn is_excluded(&self, name: &str, is_small_unit: bool) -> bool {
        let candidates = lookup_candidates(name);
        if candidates.iter().any(|c| self.exclude.contains(c)) {
            return true;
        }
        if is_small_unit && !self.exclude_small_units.is_empty() {
            let norm = normalize_text(name);
            let base = norm.split([',', '(']).next().unwrap_or(&norm);
            let bypassed = !self.exclude_small_units_except.is_empty()
                && (candidates
                    .iter()
                    .any(|c| self.exclude_small_units_except.contains(c))
                    || base
                        .split_whitespace()
                        .any(|tok| self.exclude_small_units_except.contains(tok)));
            if !bypassed
                && (candidates
                    .iter()
                    .any(|c| self.exclude_small_units.contains(c))
                    || base
                        .split_whitespace()
                        .any(|tok| self.exclude_small_units.contains(tok)))
            {
                return true;
            }
        }
        false
    }

    /// Convert a volumetric `(quantity, unit)` measurement of `name` to rounded grams.
    /// Returns `None` when unmapped, already weight-based, non-volume, non-positive, or excluded.
    pub fn to_grams(&self, quantity: f64, unit: &str, name: &str) -> Option<i64> {
        if self.grams_per_ml.is_empty() || !quantity.is_finite() || quantity <= 0.0 {
            return None;
        }
        let (ml, parsed_unit) = volume_to_ml(&self.converter, quantity, unit)?;
        let is_small = matches!(parsed_unit.symbol(), "tsp" | "tbsp");
        if self.is_excluded(name, is_small) {
            return None;
        }
        for cand in lookup_candidates(name) {
            if let Some(&density) = self.grams_per_ml.get(&cand) {
                // Snap 1e-7 float truncation artifacts from bundled unit ratios (e.g. tbsp/cup = 1/16)
                let snapped = ((ml * density) * 1e5).round() / 1e5;
                let grams = snapped.round() as i64;
                return (grams > 0).then_some(grams);
            }
        }
        None
    }
}

fn normalize_for_relpath(path: &Path) -> PathBuf {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    crate::watch::normalize_path(&abs)
}

/// Compute a Markdown relative path (always `/` separators) from `from_file`'s directory to `to_file`.
/// Returns `None` when `from_file` and `to_file` refer to the same file (self-link exclusion).
pub fn relative_markdown_path(from_file: &Path, to_file: &Path) -> Option<String> {
    let norm_from = normalize_for_relpath(from_file);
    let norm_to = normalize_for_relpath(to_file);
    if norm_from == norm_to {
        return None;
    }

    let from_dir = norm_from.parent().unwrap_or_else(|| Path::new(""));
    let from_comps: Vec<Component<'_>> = from_dir.components().collect();
    let to_comps: Vec<Component<'_>> = norm_to.components().collect();

    let common = from_comps
        .iter()
        .zip(to_comps.iter())
        .take_while(|(a, b)| a == b)
        .count();

    // Incompatible prefixes (e.g., different Windows drive letters)
    if common == 0
        && (matches!(from_comps.first(), Some(Component::Prefix(_)))
            || matches!(to_comps.first(), Some(Component::Prefix(_))))
    {
        return None;
    }

    let mut segments = Vec::new();
    for comp in &from_comps[common..] {
        if matches!(comp, Component::Normal(_)) {
            segments.push("..".to_string());
        }
    }
    for comp in &to_comps[common..] {
        if let Component::Normal(part) = comp {
            segments.push(part.to_string_lossy().to_string());
        }
    }

    (!segments.is_empty()).then(|| segments.join("/"))
}

fn dir_prefix_priority(rel_dir: &Path, link_dirs: &[String]) -> Option<usize> {
    if link_dirs.is_empty() {
        return Some(0);
    }
    let rel_norm: Vec<String> = rel_dir
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().to_string()),
            _ => None,
        })
        .collect();

    for (idx, raw_prefix) in link_dirs.iter().enumerate() {
        let prefix_parts: Vec<&str> = raw_prefix
            .trim()
            .trim_matches(|c| c == '/' || c == '\\')
            .split(['/', '\\'])
            .filter(|s| !s.is_empty() && *s != ".")
            .collect();
        if prefix_parts.is_empty() {
            return Some(idx);
        }
        if rel_norm.len() >= prefix_parts.len()
            && rel_norm
                .iter()
                .zip(prefix_parts.iter())
                .all(|(a, b)| a == b)
        {
            return Some(idx);
        }
    }
    None
}

/// Discovered recipe lookup index for internal Markdown cross-linking (`#70`).
#[derive(Debug, Clone, Default)]
pub struct RecipeLinkIndex {
    entries: HashMap<String, PathBuf>,
}

impl RecipeLinkIndex {
    /// Build a lookup index of discovered recipes under `config.recipe_search_root`,
    /// optionally restricted to directories matching `config.link_dirs`.
    pub fn build(config: &Config) -> Self {
        let search_root = crate::resolve_search_root(&config.recipe_search_root);
        if !search_root.is_dir() {
            return Self::default();
        }
        let canon_root = search_root
            .canonicalize()
            .unwrap_or_else(|_| search_root.clone());

        let discovered = crate::cook::find_recipe_file_paths(
            &search_root.to_string_lossy(),
            &config.ignore_file,
        );

        let mut ranked: Vec<(usize, PathBuf)> = Vec::new();
        for recipe_path in discovered {
            let canon_recipe = recipe_path
                .canonicalize()
                .unwrap_or_else(|_| recipe_path.clone());
            let rel_dir = canon_recipe
                .strip_prefix(&canon_root)
                .ok()
                .and_then(Path::parent)
                .unwrap_or_else(|| Path::new(""));
            if let Some(priority) = dir_prefix_priority(rel_dir, &config.link_dirs) {
                ranked.push((priority, recipe_path));
            }
        }

        // Preserve link_dirs priority first, then lexicographical path order for determinism
        ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        let mut entries = HashMap::new();
        for (_, recipe_path) in ranked {
            let recipe_str = recipe_path.to_string_lossy();
            let recipe_name = crate::cook::get_recipe_name(&recipe_str);
            if recipe_name.is_empty() {
                continue;
            }
            let recipe_slug = crate::cook::slugify_filename(&recipe_name);
            let output_path = crate::cook::get_new_recipe_path(&recipe_str, &recipe_name, config);

            let norm_name = normalize_text(&recipe_name);
            if !norm_name.is_empty() {
                entries
                    .entry(norm_name)
                    .or_insert_with(|| output_path.clone());
            }
            if !recipe_slug.is_empty() {
                entries
                    .entry(recipe_slug.clone())
                    .or_insert_with(|| output_path.clone());
                let spaced = recipe_slug.replace('-', " ");
                if !spaced.is_empty() {
                    entries.entry(spaced).or_insert_with(|| output_path.clone());
                }
            }
        }

        Self { entries }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Resolve `name` to a relative Markdown link `[name](relative/path.md)` if found
    /// (excluding self-links), otherwise return plain `name`.
    pub fn link(&self, name: &str, current_output_path: &Path) -> String {
        if self.entries.is_empty() || name.trim().is_empty() {
            return name.to_string();
        }
        let target = self.lookup(name);
        match target.and_then(|p| relative_markdown_path(current_output_path, p)) {
            Some(rel) => format!("[{name}]({rel})"),
            None => name.to_string(),
        }
    }

    fn lookup(&self, name: &str) -> Option<&Path> {
        for cand in lookup_candidates(name) {
            if let Some(path) = self.entries.get(&cand) {
                return Some(path.as_path());
            }
            let slug = crate::cook::slugify_filename(&cand);
            if let Some(path) = self.entries.get(&slug) {
                return Some(path.as_path());
            }
        }
        None
    }
}

/// Shared read-only resources loaded once per render batch and shared across worker threads.
#[derive(Debug, Default)]
pub struct RenderResources {
    pub emoji: EmojiTable,
    pub weights: WeightsTable,
    pub links: RecipeLinkIndex,
}

impl RenderResources {
    pub fn from_config(config: &Config) -> Self {
        let emoji = match EmojiTable::load(config) {
            Ok(t) => t,
            Err(e) => {
                log::warn!("Failed loading emoji mappings: {e}");
                EmojiTable::default()
            }
        };
        let weights = match WeightsTable::load(&config.weights_file) {
            Ok(t) => t,
            Err(e) => {
                log::warn!("Failed loading weights file '{}': {e}", config.weights_file);
                WeightsTable::default()
            }
        };
        let links = RecipeLinkIndex::build(config);
        Self {
            emoji,
            weights,
            links,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(prefix: &str) -> Self {
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!(
                "cook_docs_enrich_{}_{}_{}",
                prefix,
                std::process::id(),
                id
            ));
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn test_emoji_table_yaml_list_and_toml_map_formats() {
        let dir = TempDir::new("emoji");
        let yaml_path = dir.0.join("emoji.yaml");
        fs::write(
            &yaml_path,
            "\
---
emoji:
  ingredients:
    - herb:
        - cilantro
        - fresh parsley
    - tomato:
        - cherry tomatoes
  cookware:
    - bowl_with_spoon:
        - bowl
        - 215
    - \"🍳\":
        - skillet
",
        )
        .unwrap();

        let cfg = Config {
            emoji_file: yaml_path.to_string_lossy().to_string(),
            emoji: Some(serde_json::json!({
                "ingredients": {
                    ":butter:": ["butter"]
                }
            })),
            ..Config::default()
        };
        let table = EmojiTable::load(&cfg).unwrap();
        assert!(!table.is_empty());
        assert_eq!(table.lookup_ingredient("cilantro"), Some(":herb:"));
        assert_eq!(
            table.lookup_ingredient("fresh cilantro, chopped"),
            Some(":herb:")
        );
        assert_eq!(table.lookup_ingredient("cherry tomato"), Some(":tomato:"));
        assert_eq!(table.lookup_ingredient("unsalted butter"), Some(":butter:"));
        assert_eq!(table.lookup_cookware("bowl"), Some(":bowl_with_spoon:"));
        assert_eq!(table.lookup_cookware("bowls"), Some(":bowl_with_spoon:"));
        assert_eq!(table.lookup_cookware("skillet"), Some("🍳"));
        assert_eq!(table.lookup_any("skillet"), Some("🍳"));
        assert_eq!(table.lookup_ingredient("unmapped_xyz"), None);

        // TOML map format
        let toml_path = dir.0.join("emoji.toml");
        fs::write(
            &toml_path,
            "\
[ingredients]
garlic = [\"garlic\", \"garlic cloves\"]

[cookware]
pot = \"dutch oven\"
",
        )
        .unwrap();
        let cfg_toml = Config {
            emoji_file: toml_path.to_string_lossy().to_string(),
            ..Config::default()
        };
        let t2 = EmojiTable::load(&cfg_toml).unwrap();
        assert_eq!(t2.lookup_ingredient("garlic"), Some(":garlic:"));
        assert_eq!(t2.lookup_cookware("dutch oven"), Some(":pot:"));
    }

    #[test]
    fn test_emoji_table_titles_and_categories_lookup() {
        // Leading emoji detection
        assert!(has_leading_emoji(":pie: Lemon Tart"));
        assert!(has_leading_emoji("  :fork_and_knife: Dinner"));
        assert!(has_leading_emoji("🍕 Margherita"));
        assert!(has_leading_emoji("☕ Espresso"));
        assert!(!has_leading_emoji("Éclair"));
        assert!(!has_leading_emoji("Classic Lemon Tart"));
        assert!(!has_leading_emoji("Note: Soup"));
        assert!(!has_leading_emoji(":: Soup"));

        // YAML list-of-maps format for titles & categories
        let yaml_table = EmojiTable::from_value(&serde_json::json!({
            "emoji": {
                "titles": [
                    { "pizza": ["pizza", "calzone"] },
                    { "shallow_pan_of_food": ["stir-fry"] },
                    { "bread": ["bread"] }
                ],
                "ingredients": [
                    { "banana": ["banana"] },
                    { "shrimp": ["shrimp"] }
                ],
                "categories": [
                    { "cake": ["desserts", "desserts/pie"] },
                    { "birthday": ["desserts/cake"] }
                ]
            }
        }))
        .unwrap();

        // Explicit leading emoji -> no-op
        assert_eq!(
            yaml_table.lookup_title(":pie: Lemon Tart", &["desserts".to_string()]),
            None
        );
        assert_eq!(
            yaml_table.lookup_title("🍕 Pepperoni Pizza", &["mains".to_string()]),
            None
        );
        // Title word/phrase match prefers titles over ingredients and head noun
        assert_eq!(
            yaml_table.lookup_title("Pepperoni Pizza", &[]),
            Some(":pizza:")
        );
        assert_eq!(
            yaml_table.lookup_title("Banana Bread", &[]),
            Some(":bread:")
        );
        // Hyphenated title token match
        assert_eq!(
            yaml_table.lookup_title("Vegetable Stir-Fry", &[]),
            Some(":shallow_pan_of_food:")
        );
        // Falls back to ingredients word match when not in titles
        assert_eq!(
            yaml_table.lookup_title("Garlic Butter Shrimp", &[]),
            Some(":shrimp:")
        );
        // Nested category fallback prefers specific path over parent
        assert_eq!(
            yaml_table.lookup_title(
                "Grandma Special",
                &["desserts".to_string(), "cake".to_string()]
            ),
            Some(":birthday:")
        );
        assert_eq!(
            yaml_table.lookup_title(
                "House Custard",
                &["desserts".to_string(), "custard".to_string()]
            ),
            Some(":cake:")
        );
        assert_eq!(
            yaml_table.lookup_title("Mystery Dish", &["unknown".to_string()]),
            None
        );

        // Direct map format: scalar category->emoji, emoji->list, explicit emoji key, title->emoji
        let map_table = EmojiTable::from_value(&serde_json::json!({
            "titles": {
                "ramen": ":ramen:",
                "taco": ["taco", "tacos"]
            },
            "categories": {
                "breads": "bread",
                "soups": "🍲",
                "desserts/cake": ":birthday:",
                ":salad:": "salads",
                "cookie": ["cookies-and-bars", "biscuits"]
            }
        }))
        .unwrap();
        assert_eq!(map_table.lookup_title("Spicy Ramen", &[]), Some(":ramen:"));
        assert_eq!(map_table.lookup_title("Street Tacos", &[]), Some(":taco:"));
        assert_eq!(
            map_table.lookup_category(&["breads".to_string()]),
            Some(":bread:")
        );
        assert_eq!(
            map_table.lookup_category(&["soups".to_string()]),
            Some("🍲")
        );
        assert_eq!(
            map_table.lookup_category(&["salads".to_string()]),
            Some(":salad:")
        );
        assert_eq!(
            map_table.lookup_category(&["cookies-and-bars".to_string()]),
            Some(":cookie:")
        );
        assert_eq!(
            map_table.lookup_category(&["desserts/cake".to_string()]),
            Some(":birthday:")
        );
        // Invalid shapes return errors
        assert!(EmojiTable::from_value(&serde_json::json!({ "titles": 42 })).is_err());
        assert!(EmojiTable::from_value(&serde_json::json!({ "categories": 42 })).is_err());
        assert!(EmojiTable::from_value(&serde_json::json!({ "categories": [123] })).is_err());
    }

    #[test]
    fn test_weights_table_conversions_exclusions_and_formats() {
        let dir = TempDir::new("weights");
        let weights_path = dir.0.join("weights.yaml");
        fs::write(
            &weights_path,
            "\
exclude:
  - watercress
exclude_small_units:
  - salt
  - cinnamon
weights:
  all-purpose flour:
    grams_per_cup: 120
    aliases:
      - ap flour
      - flour
  granulated sugar: 200
  olive oil:
    grams_per_ml: 0.915
  honey:
    grams: 21
    volume: \"1 tbsp\"
  salt: 288
  ground cinnamon: 125
  watercress: 34
",
        )
        .unwrap();

        let table = WeightsTable::load(&weights_path.to_string_lossy()).unwrap();
        assert!(!table.is_empty());
        assert_eq!(table.to_grams(1.0, "cup", "all-purpose flour"), Some(120));
        assert_eq!(table.to_grams(2.0, "cups", "AP flour"), Some(240));
        assert_eq!(table.to_grams(1.0, "tbsp", "flour"), Some(8));
        assert_eq!(table.to_grams(1.0, "tsp", "flour"), Some(3));
        assert_eq!(table.to_grams(8.0, "fl oz", "granulated sugar"), Some(200));
        assert_eq!(table.to_grams(100.0, "ml", "olive oil"), Some(92));
        assert_eq!(table.to_grams(2.0, "Tbsp", "honey"), Some(42));
        assert_eq!(
            table.to_grams(1.0, "cup", "sifted all-purpose flour, divided"),
            Some(120)
        );

        // Weight-based units, unknown units, zero qty, unmapped, and excluded return None
        assert_eq!(table.to_grams(100.0, "g", "all-purpose flour"), None);
        assert_eq!(table.to_grams(4.0, "oz", "all-purpose flour"), None);
        assert_eq!(table.to_grams(2.0, "cloves", "garlic"), None);
        assert_eq!(table.to_grams(0.0, "cup", "flour"), None);
        assert_eq!(table.to_grams(1.0, "cup", "unknown item"), None);
        assert_eq!(table.to_grams(1.0, "cup", "watercress"), None);
        assert_eq!(table.to_grams(1.0, "tsp", "kosher salt"), None);
        assert_eq!(table.to_grams(1.0, "tbsp", "ground cinnamon"), None);
        // Large volume unit of exclude_small_units still converts
        assert_eq!(table.to_grams(1.0, "cup", "salt"), Some(288));
    }

    #[test]
    fn test_weights_exclude_small_units_except_guard() {
        let dir = TempDir::new("weights_except");
        let weights_path = dir.0.join("weights.yaml");
        fs::write(
            &weights_path,
            "\
exclude:
  - chili oil
exclude_small_units:
  - basil
  - cinnamon
  - ginger
  - salt
exclude_small_units_except:
  - pesto
  - oil
  - sugar
  - crystallized
weights:
  basil pesto: 224
  fresh basil: 24
  cinnamon sugar: 192
  ground cinnamon: 125
  crystallized ginger: 160
  fresh ginger: 96
  salt: 288
  chili oil: 216
",
        )
        .unwrap();

        let table = WeightsTable::load(&weights_path.to_string_lossy()).unwrap();
        // Compound ingredients matching exclude_small_units_except convert on small units
        assert_eq!(table.to_grams(2.0, "Tbsp", "basil pesto"), Some(28));
        assert_eq!(table.to_grams(1.0, "tsp", "cinnamon sugar"), Some(4));
        assert_eq!(table.to_grams(1.0, "tbsp", "crystallized ginger"), Some(10));
        // Plain herbs/spices without exception token remain excluded on tsp/tbsp
        assert_eq!(table.to_grams(2.0, "tbsp", "fresh basil"), None);
        assert_eq!(table.to_grams(1.0, "tsp", "ground cinnamon"), None);
        assert_eq!(table.to_grams(1.0, "tsp", "fresh ginger"), None);
        assert_eq!(table.to_grams(1.0, "tsp", "kosher salt"), None);
        // Explicit exclude still wins over exclude_small_units_except
        assert_eq!(table.to_grams(1.0, "tbsp", "chili oil"), None);
        // Large volume unit still converts
        assert_eq!(table.to_grams(1.0, "cup", "fresh basil"), Some(24));

        // Kebab-case and camelCase schema aliases also work
        let kebab = WeightsTable::from_value(&serde_json::json!({
            "exclude-small-units": ["basil"],
            "exclude-small-units-except": ["pesto"],
            "basil pesto": 224,
            "fresh basil": 24
        }))
        .unwrap();
        assert_eq!(kebab.to_grams(2.0, "tbsp", "basil pesto"), Some(28));
        assert_eq!(kebab.to_grams(2.0, "tbsp", "fresh basil"), None);

        let camel = WeightsTable::from_value(&serde_json::json!({
            "excludeSmallUnits": ["cinnamon"],
            "excludeSmallUnitsExcept": ["sugar"],
            "cinnamon sugar": 192,
            "ground cinnamon": 125
        }))
        .unwrap();
        assert_eq!(camel.to_grams(1.0, "tsp", "cinnamon sugar"), Some(4));
        assert_eq!(camel.to_grams(1.0, "tsp", "ground cinnamon"), None);
    }

    #[test]
    fn test_recipe_link_index_nested_paths_and_self_exclusion() {
        let dir = TempDir::new("links");
        let cook_dir = dir.0.join("cook");
        let out_dir = dir.0.join("docs");
        fs::create_dir_all(cook_dir.join("mains")).unwrap();
        fs::create_dir_all(cook_dir.join("sauces-and-dressings")).unwrap();
        fs::create_dir_all(cook_dir.join("desserts")).unwrap();

        fs::write(
            cook_dir.join("mains/Pizza.cook"),
            "Spread @pizza sauce{1%cup}.",
        )
        .unwrap();
        fs::write(
            cook_dir.join("sauces-and-dressings/Pizza Sauce.cook"),
            "Simmer @tomatoes{2%cups}.",
        )
        .unwrap();
        fs::write(cook_dir.join("desserts/Cake.cook"), "Bake @flour{1%cup}.").unwrap();

        let cfg = Config {
            recipe_search_root: cook_dir.to_string_lossy().to_string(),
            output_dir: out_dir.to_string_lossy().to_string(),
            ignore_file: String::new(),
            link_dirs: vec!["sauces-and-dressings".to_string()],
            ..Config::default()
        };

        let index = RecipeLinkIndex::build(&cfg);
        assert!(!index.is_empty());

        let pizza_out = out_dir.join("mains/pizza.md");
        let sauce_out = out_dir.join("sauces-and-dressings/pizza-sauce.md");

        // Nested cross-directory link
        assert_eq!(
            index.link("pizza sauce", &pizza_out),
            "[pizza sauce](../sauces-and-dressings/pizza-sauce.md)"
        );
        // Self-link excluded
        assert_eq!(index.link("pizza sauce", &sauce_out), "pizza sauce");
        // Recipe outside link_dirs not linked
        assert_eq!(index.link("cake", &pizza_out), "cake");
        // Unmatched ingredient stays plain
        assert_eq!(index.link("olive oil", &pizza_out), "olive oil");

        // Empty link_dirs indexes all recipes
        let cfg_all = Config {
            link_dirs: Vec::new(),
            ..cfg
        };
        let index_all = RecipeLinkIndex::build(&cfg_all);
        assert_eq!(
            index_all.link("cake", &pizza_out),
            "[cake](../desserts/cake.md)"
        );
    }

    #[test]
    fn test_weights_and_emoji_error_cases() {
        let dir = TempDir::new("errs");
        assert!(WeightsTable::load("/nonexistent/weights.yaml").is_err());

        let bad_ext = dir.0.join("w.txt");
        fs::write(&bad_ext, "flour: 120").unwrap();
        assert!(WeightsTable::load(&bad_ext.to_string_lossy()).is_err());

        let neg = dir.0.join("neg.yaml");
        fs::write(&neg, "flour: -5\n").unwrap();
        assert!(WeightsTable::load(&neg.to_string_lossy()).is_err());

        let bad_unit = dir.0.join("unit.yaml");
        fs::write(&bad_unit, "flour:\n  grams: 100\n  volume: \"1 pinch\"\n").unwrap();
        assert!(WeightsTable::load(&bad_unit.to_string_lossy()).is_err());

        assert!(EmojiTable::from_value(&serde_json::json!(42)).is_err());
        assert!(EmojiTable::from_value(&serde_json::json!({ "ingredients": [123] })).is_err());
    }
}

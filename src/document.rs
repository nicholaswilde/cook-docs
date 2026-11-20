use crate::types::Recipe;
use gtmpl_ng::{FuncError, Template, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_DOCUMENTATION_TEMPLATE: &str = r#"{{ template "cook.headerSection" . }}
 
{{ template "cook.lazyImageSection" . }}
 
{{ template "cook.tableSection" . }}
 
{{ template "cook.ingredientsSection" . }}
 
{{ template "cook.cookwareSection" . }}
 
{{ template "cook.stepsSection" . -}}
 
{{ template "cook.sourceSection" . }}
"#;

fn get_header_template() -> String {
    r#"{{ define "cook.headerSection" }}# {{ .Info.RecipeName }}{{ end }}"#.to_string()
}

fn get_image_template() -> String {
    r#"{{ define "cook.imageSection" }}{{ if .Info.ImageFileName }}![{{ .Info.RecipeName }}](../assets/images/{{ lower .Info.ImageFileName | replace " " "-" }}){{ end }}{{ end }}"#.to_string()
}

fn get_lazy_image_template() -> String {
    r#"{{ define "cook.lazyImageSection" }}{{ if .Info.ImageFileName }}![{{ .Info.RecipeName }}](../assets/images/{{ lower .Info.ImageFileName | replace " " "-" }}){ loading=lazy }{{ end }}{{ end }}"#.to_string()
}

fn get_table_template() -> String {
    r#"{{ define "cook.tableSection" }}{{ if or .Metadata.servings .Metadata.serves }}| :fork_and_knife_with_plate: Serves | :timer_clock: Total Time |
|:----------------------------------:|:-----------------------: |
| {{ if .Metadata.servings }}{{ .Metadata.servings }}{{ else if .Metadata.serves }}{{ .Metadata.serves }}{{ end }} | {{ sumTimers .Steps }} |{{ else }}| :timer_clock: Total Time |
|:-----------------------: |
| {{ sumTimers .Steps }} |{{ end }}{{ end }}"#.to_string()
}

fn get_ingredients_template() -> String {
    let mut s = String::new();
    s.push_str(r#"{{ define "cook.ingredientsHeader" }}## :salt: Ingredients{{ end }}"#);
    s.push_str(r#"{{ define "cook.ingredients" }}{{ range .Steps }}{{- range .Ingredients }}"#);
    s.push('\n');
    s.push_str(r#"- {{ if .Amount.Quantity }}{{ round .Amount.Quantity 2 }}{{ if .Amount.Unit }} {{ .Amount.Unit }}{{ end }}{{ else }}some{{ end }} {{ .Name }}{{- end }}{{- end }}{{ end }}"#);
    s.push_str(
        r#"{{ define "cook.ingredientsSection" }}{{ template "cook.ingredientsHeader" . }}"#,
    );
    s.push('\n');
    s.push_str(r#"{{ template "cook.ingredients" . }}{{ end }}"#);
    s
}

fn get_cookware_template() -> String {
    let mut s = String::new();
    s.push_str(r#"{{ define "cook.cookwareHeader" }}## :cooking: Cookware{{ end }}"#);
    s.push_str(r#"{{ define "cook.cookware" }}{{ range .Steps }}{{- range .Cookware }}"#);
    s.push('\n');
    s.push_str(r#"- {{.Quantity}} {{.Name}}{{- end }}{{- end }}{{ end }}"#);
    s.push_str(r#"{{ define "cook.cookwareSection" }}{{ template "cook.cookwareHeader" . }}"#);
    s.push('\n');
    s.push_str(r#"{{ template "cook.cookware" . }}{{ end }}"#);
    s
}

fn get_steps_template() -> String {
    let mut s = String::new();
    s.push_str(r#"{{ define "cook.stepsHeader" }}## :pencil: Instructions{{ end }}"#);
    s.push_str(r#"{{ define "cook.steps" }}{{ range $i, $a := .Steps }}"#);
    s.push_str("\n\n### Step {{add1 $i}}\n\n");
    s.push_str(r#"{{ wrap $.Config.WordWrap .Directions }}{{- end }}{{ end }}"#);
    s.push_str(r#"{{ define "cook.stepsSection" }}{{ template "cook.stepsHeader" . }}{{ template "cook.steps" . }}{{ end }}"#);
    s
}

fn get_steps_with_quoted_comments_template() -> String {
    let mut s = String::new();
    s.push_str(
        r#"{{ define "cook.stepsWithQuotedCommentsHeader" }}## :pencil: Instructions{{ end }}"#,
    );
    s.push_str(r#"{{ define "cook.stepsWithQuotedComments" }}{{ range $i, $a := .Steps }}"#);
    s.push_str("\n\n### Step {{add1 $i}}\n\n");
    s.push_str(r#"{{ wrap $.Config.WordWrap .Directions }}"#);
    s.push_str("\n\n");
    s.push_str(r#"{{ range .Comments }}"#);
    s.push_str("\n> ");
    s.push_str(r#"{{.}}{{- end }}{{- end }}{{ end }}"#);
    s.push_str(r#"{{ define "cook.stepsWithQuotedCommentsSection" }}{{ template "cook.stepsWithQuotedCommentsHeader" . }}{{ template "cook.stepsWithQuotedComments" . }}{{ end }}"#);
    s
}

fn get_steps_with_admonished_comments_template() -> String {
    let mut s = String::new();
    s.push_str(
        r#"{{ define "cook.stepsWithAdmonishedCommentsHeader" }}## :pencil: Instructions{{ end }}"#,
    );
    s.push_str(r#"{{ define "cook.stepsWithAdmonishedComments" }}{{ range $i, $a := .Steps }}"#);
    s.push_str("\n\n### Step {{add1 $i}}\n\n");
    s.push_str(r#"{{ wrap $.Config.WordWrap .Directions }}"#);
    s.push_str("\n\n");
    s.push_str(r#"{{ range .Comments }}"#);
    s.push_str("\n!!! note\n");
    s.push_str(r#"{{ indent 6 . }}{{- end }}{{- end }}{{ end }}"#);
    s.push_str(r#"{{ define "cook.stepsWithAdmonishedCommentsSection" }}{{ template "cook.stepsWithAdmonishedCommentsHeader" . }}{{ template "cook.stepsWithAdmonishedComments" . }}{{ end }}"#);
    s
}

fn get_source_template() -> String {
    let mut s = String::new();
    s.push_str(r#"{{ define "cook.sourceHeader" }}## :link: Source{{ end }}"#);
    s.push_str(r#"{{- define "cook.source" }}- {{ getSource .Metadata.source }}{{ end }}"#);
    s.push_str(r#"{{ define "cook.sourceSection" }}{{ if .Metadata.source }}"#);
    s.push_str("\n\n");
    s.push_str(r#"{{ template "cook.sourceHeader" . }}"#);
    s.push_str("\n\n");
    s.push_str(r#"{{ template "cook.source" . }}{{ end }}{{ end }}"#);
    s
}

fn get_metadata_template() -> String {
    let mut s = String::new();
    s.push_str(r#"{{ define "cook.metadataHeader" }}## Metadata{{ end }}"#);
    s.push_str(r#"{{ define "cook.metadata" }}{{ range $key, $value := .Metadata }}"#);
    s.push_str(r#"\n- {{ $key }}: {{ $value }}{{ end }}{{ end }}"#);
    s.push_str(r#"{{ define "cook.metadataSection" }}{{ template "cook.metadataHeader" . }}"#);
    s.push('\n');
    s.push_str(r#"{{ template "cook.metadata" . }}{{ end }}"#);
    s
}

fn get_comments_template() -> String {
    let mut s = String::new();
    s.push_str(r#"{{ define "cook.commentsHeader" }}## Comments{{ end }}"#);
    s.push_str(r#"{{ define "cook.comments" }}{{ range .Steps }}{{ range .Comments }}"#);
    s.push_str(r#"\n- {{.}}{{- end }}{{- end }}{{ end }}"#);
    s.push_str(r#"{{ define "cook.commentsSection" }}{{ template "cook.commentsHeader" . }}"#);
    s.push('\n');
    s.push_str(r#"{{ template "cook.comments" . }}{{ end }}"#);
    s
}

fn get_source(source: &str) -> String {
    if (source.starts_with("http://") || source.starts_with("https://")) && !source.contains(' ') {
        format!("<{}>", source)
    } else {
        source.to_string()
    }
}

fn value_to_f64(v: &Value) -> f64 {
    if let Value::Number(n) = v {
        if let Some(f) = n.as_f64() {
            f
        } else if let Some(i) = n.as_i64() {
            i as f64
        } else if let Some(u) = n.as_u64() {
            u as f64
        } else {
            0.0
        }
    } else {
        0.0
    }
}

fn get_source_fn(args: &[Value]) -> Result<Value, FuncError> {
    if args.is_empty() {
        return Ok(Value::String(String::new()));
    }
    let source = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => "",
    };
    Ok(Value::String(get_source(source)))
}

fn sum_timers(args: &[Value]) -> Result<Value, FuncError> {
    if args.is_empty() {
        return Ok(Value::String("0 minutes".to_string()));
    }
    let steps = match &args[0] {
        Value::Array(arr) => arr,
        _ => {
            return Err(FuncError::Generic(
                "sumTimers expects an array of steps".to_string(),
            ));
        }
    };

    let mut sum = 0.0;
    for step in steps {
        if let Some(Value::Array(timers)) = match step {
            Value::Map(m) => m.get("Timers"),
            _ => None,
        } {
            for timer in timers {
                if let Value::Map(timer_map) = timer {
                    let duration = match timer_map.get("Duration") {
                        Some(v) => value_to_f64(v),
                        _ => 0.0,
                    };
                    let unit = match timer_map.get("Unit") {
                        Some(Value::String(s)) => s.as_str(),
                        _ => "",
                    };
                    match unit {
                        "day" | "days" => {
                            sum += duration * 60.0 * 24.0;
                        }
                        "hour" | "hours" => {
                            sum += duration * 60.0;
                        }
                        "minute" | "minutes" => {
                            sum += duration;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let res = if sum > 1440.0 {
        format!("{:.2} days", sum / 1440.0)
    } else if sum > 60.0 {
        format!("{:.2} hours", sum / 60.0)
    } else {
        format!("{:.0} minutes", sum)
    };
    Ok(Value::String(res))
}

fn lower(args: &[Value]) -> Result<Value, FuncError> {
    if args.is_empty() {
        return Ok(Value::String(String::new()));
    }
    let s = match &args[0] {
        Value::String(s) => s.to_lowercase(),
        _ => String::new(),
    };
    Ok(Value::String(s))
}

fn replace(args: &[Value]) -> Result<Value, FuncError> {
    if args.len() < 3 {
        return Ok(Value::String(String::new()));
    }
    let old = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => "",
    };
    let new = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => "",
    };
    let input = match &args[2] {
        Value::String(s) => s.as_str(),
        _ => "",
    };
    Ok(Value::String(input.replace(old, new)))
}

fn round(args: &[Value]) -> Result<Value, FuncError> {
    if args.is_empty() {
        return Ok(Value::String(String::new()));
    }
    let val = value_to_f64(&args[0]);
    let decimals = if args.len() > 1 {
        match &args[1] {
            Value::Number(n) => n.as_i64().unwrap_or(0) as usize,
            _ => 0,
        }
    } else {
        0
    };
    let formatted = format!("{:.*}", decimals, val);
    let rounded_val = formatted.parse::<f64>().unwrap_or(val);
    Ok(Value::Number(rounded_val.into()))
}

fn add1(args: &[Value]) -> Result<Value, FuncError> {
    if args.is_empty() {
        return Ok(Value::Number(1.into()));
    }
    let val = match &args[0] {
        Value::Number(n) => n.as_i64().unwrap_or(0),
        _ => 0,
    };
    Ok(Value::Number((val + 1).into()))
}

fn wrap(args: &[Value]) -> Result<Value, FuncError> {
    if args.len() < 2 {
        return Ok(args
            .first()
            .cloned()
            .unwrap_or(Value::String(String::new())));
    }
    let limit = match &args[0] {
        Value::Number(n) => n.as_i64().unwrap_or(120) as usize,
        _ => 120,
    };
    let text = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => "",
    };

    let mut result = String::new();
    for paragraph in text.split('\n') {
        let mut current_line = String::new();
        for word in paragraph.split_whitespace() {
            if current_line.is_empty() {
                current_line.push_str(word);
            } else if current_line.len() + 1 + word.len() > limit {
                result.push_str(&current_line);
                result.push('\n');
                current_line = word.to_string();
            } else {
                current_line.push(' ');
                current_line.push_str(word);
            }
        }
        result.push_str(&current_line);
        result.push('\n');
    }
    if !text.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    Ok(Value::String(result))
}

fn indent(args: &[Value]) -> Result<Value, FuncError> {
    if args.len() < 2 {
        return Ok(Value::String(String::new()));
    }
    let spaces_count = match &args[0] {
        Value::Number(n) => n.as_i64().unwrap_or(0) as usize,
        _ => 0,
    };
    let text = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => "",
    };
    let padding = " ".repeat(spaces_count);
    let mut result = String::new();
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            result.push('\n');
        }
        if !line.is_empty() {
            result.push_str(&padding);
            result.push_str(line);
        }
    }
    Ok(Value::String(result))
}

fn json_to_gtmpl(val: serde_json::Value) -> Value {
    match val {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(num) => {
            if let Some(i) = num.as_i64() {
                Value::Number(i.into())
            } else if let Some(u) = num.as_u64() {
                Value::Number((u as i64).into())
            } else if let Some(f) = num.as_f64() {
                Value::Number(f.into())
            } else {
                Value::Nil
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            let mapped = arr.into_iter().map(json_to_gtmpl).collect();
            Value::Array(mapped)
        }
        serde_json::Value::Object(obj) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_gtmpl(v));
            }
            Value::Map(map)
        }
    }
}

fn apply_markdown_format(input: &str) -> String {
    let s = input.replace(" \n", "\n");
    let mut result = String::new();
    let mut newline_count = 0;
    for c in s.chars() {
        if c == '\n' {
            newline_count += 1;
            if newline_count <= 2 {
                result.push(c);
            }
        } else {
            newline_count = 0;
            result.push(c);
        }
    }
    result
}

/// Resolve a single configured template entry to its on-disk path.
pub(crate) fn resolve_template_path(
    recipe_search_root: &str,
    recipe_path: &str,
    template_file: &str,
) -> PathBuf {
    if crate::cook::is_relative_path(template_file) {
        Path::new(recipe_search_root).join(template_file)
    } else if crate::cook::is_base_filename(template_file) {
        let parent = Path::new(recipe_path)
            .parent()
            .unwrap_or_else(|| Path::new(""));
        parent.join(template_file)
    } else {
        PathBuf::from(template_file)
    }
}

fn get_documentation_template(
    recipe_search_root: &str,
    recipe_path: &str,
    template_files: &[String],
) -> Result<String, String> {
    let mut template_files_for_recipe = Vec::new();
    let mut template_not_found = false;

    for template_file in template_files {
        let full_template_path =
            resolve_template_path(recipe_search_root, recipe_path, template_file);

        if !full_template_path.exists() || !full_template_path.is_file() {
            log::debug!(
                "Did not find template file {:?} for recipe {}, using default template",
                full_template_path,
                recipe_path
            );
            template_not_found = true;
            continue;
        }

        template_files_for_recipe.push(full_template_path);
    }

    log::debug!(
        "Using template files {:?} for recipe {}",
        template_files,
        recipe_path
    );
    let mut all_template_contents = String::new();

    for file_path in template_files_for_recipe {
        let contents = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read template file {:?}: {}", file_path, e))?;
        all_template_contents.push_str(&contents);
    }

    if template_not_found || template_files.is_empty() || all_template_contents.is_empty() {
        all_template_contents.push_str(DEFAULT_DOCUMENTATION_TEMPLATE);
    }

    Ok(all_template_contents)
}

fn get_documentation_templates(
    recipe_search_root: &str,
    recipe_path: &str,
    template_files: &[String],
) -> Result<Vec<String>, String> {
    let doc_template = get_documentation_template(recipe_search_root, recipe_path, template_files)?;
    Ok(vec![
        get_header_template(),
        get_image_template(),
        get_lazy_image_template(),
        get_table_template(),
        get_ingredients_template(),
        get_cookware_template(),
        get_steps_template(),
        get_steps_with_quoted_comments_template(),
        get_steps_with_admonished_comments_template(),
        get_source_template(),
        get_metadata_template(),
        get_comments_template(),
        doc_template,
    ])
}

/// Load, register helper functions, and compile documentation templates for a recipe.
pub(crate) fn build_documentation_template(
    recipe_search_root: &str,
    recipe_path: &str,
    template_files: &[String],
) -> Result<Template, String> {
    let mut doc_template = Template::default();
    doc_template.add_funcs(&[
        (
            "getSource",
            get_source_fn as fn(&[Value]) -> Result<Value, FuncError>,
        ),
        (
            "sumTimers",
            sum_timers as fn(&[Value]) -> Result<Value, FuncError>,
        ),
        ("lower", lower as fn(&[Value]) -> Result<Value, FuncError>),
        (
            "replace",
            replace as fn(&[Value]) -> Result<Value, FuncError>,
        ),
        ("round", round as fn(&[Value]) -> Result<Value, FuncError>),
        ("add1", add1 as fn(&[Value]) -> Result<Value, FuncError>),
        ("wrap", wrap as fn(&[Value]) -> Result<Value, FuncError>),
        ("indent", indent as fn(&[Value]) -> Result<Value, FuncError>),
    ]);

    let templates_list =
        get_documentation_templates(recipe_search_root, recipe_path, template_files)?;
    for t in templates_list {
        doc_template
            .parse(t)
            .map_err(|e| format!("Failed to parse template: {e}"))?;
    }

    Ok(doc_template)
}

pub fn print_documentation(recipe: &Recipe) {
    if recipe.config.jsonify {
        log::info!(
            "Printing json output for recipe {}",
            recipe.info.new_recipe_file_path
        );
        if let Ok(j) = serde_json::to_string_pretty(recipe) {
            log::info!("{}", j);
        }
        return;
    }

    log::info!(
        "Generating markdown file for recipe {}",
        recipe.info.new_recipe_file_path
    );

    let doc_template = match build_documentation_template(
        &recipe.config.recipe_search_root,
        &recipe.info.recipe_file_path,
        &recipe.config.template_files,
    ) {
        Ok(t) => t,
        Err(e) => {
            log::warn!(
                "Error preparing template for {}: {}",
                recipe.info.recipe_file_path,
                e
            );
            return;
        }
    };

    // 3. Serialize context to json value and then map to gtmpl value
    let json_val = match serde_json::to_value(recipe) {
        Ok(v) => v,
        Err(e) => {
            log::warn!(
                "Error serializing recipe to JSON for {}: {}",
                recipe.info.recipe_file_path,
                e
            );
            return;
        }
    };
    let gtmpl_context_value = json_to_gtmpl(json_val);
    let gtmpl_context = gtmpl_ng::Context::from(gtmpl_context_value);

    // 4. Render template
    let rendered = match doc_template.render(&gtmpl_context) {
        Ok(out) => out,
        Err(e) => {
            log::warn!(
                "Error executing template {}: {}",
                recipe.info.recipe_file_path,
                e
            );
            return;
        }
    };

    // 5. Apply formatting and write to output file
    let formatted = apply_markdown_format(&rendered);

    if recipe.config.dry_run {
        println!("{}", formatted);
    } else {
        let out_path = Path::new(&recipe.info.new_recipe_file_path);
        if let Err(e) = out_path.parent().map(fs::create_dir_all).unwrap_or(Ok(())) {
            log::warn!(
                "Could not create parent directory for recipe markdown file {}, skipping recipe: {}",
                recipe.info.new_recipe_file_path,
                e
            );
            return;
        }
        if let Err(e) = fs::write(out_path, formatted) {
            log::warn!(
                "Error generating documentation file for recipe {}: {}",
                recipe.info.new_recipe_file_path,
                e
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Config, Cookware, Info, Ingredient, IngredientAmount, Step, Timer};
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(prefix: &str) -> Self {
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!(
                "cook_docs_doc_{}_{}_{}",
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

    fn sample_recipe(dir: &Path, dry_run: bool, jsonify: bool) -> Recipe {
        let mut metadata = HashMap::new();
        metadata.insert("servings".to_string(), "4".to_string());
        metadata.insert(
            "source".to_string(),
            "https://example.com/recipe".to_string(),
        );

        Recipe {
            steps: vec![Step {
                directions: "Mix flour and water thoroughly for a smooth batter".to_string(),
                timers: vec![
                    Timer {
                        name: "prep".to_string(),
                        duration: 15.0,
                        unit: "minutes".to_string(),
                    },
                    Timer {
                        name: "bake".to_string(),
                        duration: 1.0,
                        unit: "hour".to_string(),
                    },
                ],
                ingredients: vec![
                    Ingredient {
                        name: "flour".to_string(),
                        amount: IngredientAmount {
                            is_numeric: true,
                            quantity: 250.256,
                            quantity_raw: "250.256".to_string(),
                            unit: "g".to_string(),
                        },
                    },
                    Ingredient {
                        name: "salt".to_string(),
                        amount: IngredientAmount {
                            is_numeric: false,
                            quantity: 0.0,
                            quantity_raw: "".to_string(),
                            unit: "".to_string(),
                        },
                    },
                ],
                cookware: vec![Cookware {
                    is_numeric: false,
                    name: "bowl".to_string(),
                    quantity: 1.0,
                    quantity_raw: "".to_string(),
                }],
                comments: vec!["Stir gently".to_string()],
            }],
            metadata,
            config: Config {
                dry_run,
                jsonify,
                ignore_file: ".cookdocsignore".to_string(),
                recipe_search_root: dir.to_string_lossy().to_string(),
                log_level: "info".to_string(),
                template_files: vec!["recipe.md.gotmpl".to_string()],
                word_wrap: 25,
                output_dir: "".to_string(),
            },
            info: Info {
                image_file_name: "My Recipe.png".to_string(),
                image_file_path: dir.join("My Recipe.png").to_string_lossy().to_string(),
                new_recipe_file_path: dir.join("my-recipe.md").to_string_lossy().to_string(),
                recipe_name: "My Recipe".to_string(),
                recipe_file_path: dir.join("My Recipe.cook").to_string_lossy().to_string(),
            },
        }
    }

    #[test]
    fn test_template_helper_functions() {
        assert_eq!(get_source("https://example.com"), "<https://example.com>");
        assert_eq!(get_source("http://example.com"), "<http://example.com>");
        assert_eq!(get_source("Grandma's book"), "Grandma's book");
        assert_eq!(
            get_source("https://ex.com with space"),
            "https://ex.com with space"
        );

        assert_eq!(get_source_fn(&[]).unwrap(), Value::String(String::new()));
        assert_eq!(
            get_source_fn(&[Value::Number(1.into())]).unwrap(),
            Value::String(String::new())
        );
        assert_eq!(
            get_source_fn(&[Value::String("https://a.test".to_string())]).unwrap(),
            Value::String("<https://a.test>".to_string())
        );

        assert_eq!(value_to_f64(&Value::Number(5i64.into())), 5.0);
        assert_eq!(value_to_f64(&Value::Number(2.5f64.into())), 2.5);
        assert_eq!(value_to_f64(&Value::Bool(true)), 0.0);

        assert_eq!(
            lower(&[Value::String("HeLLo".to_string())]).unwrap(),
            Value::String("hello".to_string())
        );
        assert_eq!(lower(&[]).unwrap(), Value::String(String::new()));

        assert_eq!(
            replace(&[
                Value::String(" ".to_string()),
                Value::String("-".to_string()),
                Value::String("a b c".to_string()),
            ])
            .unwrap(),
            Value::String("a-b-c".to_string())
        );
        assert_eq!(replace(&[]).unwrap(), Value::String(String::new()));

        assert_eq!(
            round(&[Value::Number(1.2345f64.into()), Value::Number(2i64.into())]).unwrap(),
            Value::Number(1.23f64.into())
        );
        assert_eq!(
            round(&[Value::Number(1.6f64.into())]).unwrap(),
            Value::Number(2.0f64.into())
        );
        assert_eq!(round(&[]).unwrap(), Value::String(String::new()));

        assert_eq!(add1(&[]).unwrap(), Value::Number(1i64.into()));
        assert_eq!(
            add1(&[Value::Number(4i64.into())]).unwrap(),
            Value::Number(5i64.into())
        );
    }

    #[test]
    fn test_sum_timers_units_and_errors() {
        assert_eq!(
            sum_timers(&[]).unwrap(),
            Value::String("0 minutes".to_string())
        );
        assert!(sum_timers(&[Value::Bool(false)]).is_err());

        let make_step = |d: f64, u: &str| {
            let mut tm = HashMap::new();
            tm.insert("Duration".to_string(), Value::Number(d.into()));
            tm.insert("Unit".to_string(), Value::String(u.to_string()));
            let mut step = HashMap::new();
            step.insert("Timers".to_string(), Value::Array(vec![Value::Map(tm)]));
            Value::Map(step)
        };

        assert_eq!(
            sum_timers(&[Value::Array(vec![make_step(30.0, "minutes")])]).unwrap(),
            Value::String("30 minutes".to_string())
        );
        assert_eq!(
            sum_timers(&[Value::Array(vec![make_step(2.0, "hours")])]).unwrap(),
            Value::String("2.00 hours".to_string())
        );
        assert_eq!(
            sum_timers(&[Value::Array(vec![
                make_step(2.0, "days"),
                make_step(5.0, "sec")
            ])])
            .unwrap(),
            Value::String("2.00 days".to_string())
        );
    }

    #[test]
    fn test_wrap_indent_json_and_format() {
        assert_eq!(wrap(&[]).unwrap(), Value::String(String::new()));
        assert_eq!(
            wrap(&[
                Value::Number(10i64.into()),
                Value::String("one two three four".to_string()),
            ])
            .unwrap(),
            Value::String("one two\nthree four".to_string())
        );
        assert_eq!(
            wrap(&[
                Value::Number(10i64.into()),
                Value::String("one two\nthree".to_string()),
            ])
            .unwrap(),
            Value::String("one two\nthree".to_string())
        );

        assert_eq!(indent(&[]).unwrap(), Value::String(String::new()));
        assert_eq!(
            indent(&[
                Value::Number(2i64.into()),
                Value::String("a\n\nb".to_string()),
            ])
            .unwrap(),
            Value::String("  a\n\n  b".to_string())
        );

        let mapped = json_to_gtmpl(serde_json::json!({
            "nil": null,
            "b": true,
            "i": -3,
            "u": u64::MAX,
            "f": 1.5,
            "s": "ok",
            "arr": [1, 2]
        }));
        assert!(matches!(mapped, Value::Map(_)));

        assert_eq!(apply_markdown_format("a \nb\n\n\n\nc"), "a\nb\n\nc");
    }

    #[test]
    fn test_template_loading_and_built_in_sections() {
        let dir = TempDir::new("templates");
        let recipe_path = dir.0.join("My Recipe.cook");
        fs::write(&recipe_path, "").unwrap();

        let default_tpl = get_documentation_template(
            &dir.0.to_string_lossy(),
            &recipe_path.to_string_lossy(),
            &[],
        )
        .unwrap();
        assert!(default_tpl.contains("cook.headerSection"));

        let custom_base = dir.0.join("recipe.md.gotmpl");
        fs::write(&custom_base, "CUSTOM").unwrap();
        let loaded = get_documentation_template(
            &dir.0.to_string_lossy(),
            &recipe_path.to_string_lossy(),
            &["recipe.md.gotmpl".to_string()],
        )
        .unwrap();
        assert_eq!(loaded, "CUSTOM");

        fs::create_dir_all(dir.0.join("tpl")).unwrap();
        fs::write(dir.0.join("tpl/rel.gotmpl"), "REL").unwrap();
        let rel_loaded = get_documentation_template(
            &dir.0.to_string_lossy(),
            &recipe_path.to_string_lossy(),
            &["./tpl/rel.gotmpl".to_string()],
        )
        .unwrap();
        assert_eq!(rel_loaded, "REL");

        let direct = dir.0.join("direct.gotmpl");
        fs::write(&direct, "DIRECT").unwrap();
        let direct_loaded = get_documentation_template(
            &dir.0.to_string_lossy(),
            &recipe_path.to_string_lossy(),
            &[direct.to_string_lossy().to_string()],
        )
        .unwrap();
        assert_eq!(direct_loaded, "DIRECT");

        let err = get_documentation_template(
            &dir.0.to_string_lossy(),
            &recipe_path.to_string_lossy(),
            &[dir.0.to_string_lossy().to_string()],
        );
        assert!(err.is_ok());
    }

    #[test]
    fn test_print_documentation_writes_renders_and_handles_errors() {
        let dir = TempDir::new("render");
        let recipe = sample_recipe(&dir.0, false, false);
        print_documentation(&recipe);
        let out_md = fs::read_to_string(&recipe.info.new_recipe_file_path).unwrap();
        assert!(out_md.contains("# My Recipe"));
        assert!(out_md.contains("my-recipe.png"));
        assert!(out_md.contains("250.26 g flour"));
        assert!(out_md.contains("some salt"));
        assert!(out_md.contains("1 bowl"));
        assert!(out_md.contains("1.25 hours"));
        assert!(out_md.contains("<https://example.com/recipe>"));

        let mut custom_recipe = sample_recipe(&dir.0, false, false);
        let tpl = dir.0.join("all.gotmpl");
        fs::write(
            &tpl,
            r#"{{ template "cook.imageSection" . }}
{{ template "cook.stepsWithQuotedCommentsSection" . }}
{{ template "cook.stepsWithAdmonishedCommentsSection" . }}
{{ template "cook.metadataSection" . }}
{{ template "cook.commentsSection" . }}"#,
        )
        .unwrap();
        custom_recipe.config.template_files = vec!["all.gotmpl".to_string()];
        custom_recipe.metadata.remove("servings");
        custom_recipe
            .metadata
            .insert("serves".to_string(), "2".to_string());
        print_documentation(&custom_recipe);
        let custom_md = fs::read_to_string(&custom_recipe.info.new_recipe_file_path).unwrap();
        assert!(custom_md.contains("> Stir gently"));
        assert!(custom_md.contains("!!! note\n      Stir gently"));
        assert!(custom_md.contains("## Metadata"));
        assert!(custom_md.contains("## Comments"));

        let dry = sample_recipe(&dir.0, true, false);
        print_documentation(&dry);

        let json = sample_recipe(&dir.0, false, true);
        print_documentation(&json);

        let mut bad_tpl = sample_recipe(&dir.0, false, false);
        fs::write(&tpl, "{{ invalid").unwrap();
        bad_tpl.config.template_files = vec!["all.gotmpl".to_string()];
        print_documentation(&bad_tpl);

        let mut bad_out = sample_recipe(&dir.0, false, false);
        let blocking_file = dir.0.join("blocker");
        fs::write(&blocking_file, "x").unwrap();
        bad_out.info.new_recipe_file_path =
            blocking_file.join("out.md").to_string_lossy().to_string();
        print_documentation(&bad_out);
    }
}

use std::fs;
use std::path::{Path, PathBuf};
use gtmpl_ng::{FuncError, Value, Template};
use crate::types::Recipe;

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
    s.push_str(r#"{{ define "cook.ingredientsSection" }}{{ template "cook.ingredientsHeader" . }}"#);
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
    s.push_str(r#"{{ define "cook.stepsWithQuotedCommentsHeader" }}## :pencil: Instructions{{ end }}"#);
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
    s.push_str(r#"{{ define "cook.stepsWithAdmonishedCommentsHeader" }}## :pencil: Instructions{{ end }}"#);
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
        _ => return Err(FuncError::Generic("sumTimers expects an array of steps".to_string())),
    };
    
    let mut sum = 0.0;
    for step in steps {
        if let Some(Value::Array(timers)) = match step { Value::Map(m) => m.get("Timers"), _ => None } {
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
    let old = match &args[0] { Value::String(s) => s.as_str(), _ => "" };
    let new = match &args[1] { Value::String(s) => s.as_str(), _ => "" };
    let input = match &args[2] { Value::String(s) => s.as_str(), _ => "" };
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
        return Ok(args.first().cloned().unwrap_or(Value::String(String::new())));
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

fn get_documentation_template(recipe_search_root: &str, recipe_path: &str, template_files: &[String]) -> Result<String, String> {
    let mut template_files_for_recipe = Vec::new();
    let mut template_not_found = false;

    for template_file in template_files {
        let full_template_path = if crate::cook::is_relative_path(template_file) {
            Path::new(recipe_search_root).join(template_file)
        } else if crate::cook::is_base_filename(template_file) {
            let parent = Path::new(recipe_path).parent().unwrap_or_else(|| Path::new(""));
            parent.join(template_file)
        } else {
            PathBuf::from(template_file)
        };

        if !full_template_path.exists() || !full_template_path.is_file() {
            log::debug!("Did not find template file {:?} for recipe {}, using default template", full_template_path, recipe_path);
            template_not_found = true;
            continue;
        }

        template_files_for_recipe.push(full_template_path);
    }

    log::debug!("Using template files {:?} for recipe {}", template_files, recipe_path);
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

fn get_documentation_templates(recipe_search_root: &str, recipe_path: &str, template_files: &[String]) -> Result<Vec<String>, String> {
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

pub fn print_documentation(recipe: &Recipe) {
    if recipe.config.jsonify {
        log::info!("Printing json output for recipe {}", recipe.info.new_recipe_file_path);
        if let Ok(j) = serde_json::to_string_pretty(recipe) {
            log::info!("{}", j);
        }
        return;
    }

    log::info!("Generating markdown file for recipe {}", recipe.info.new_recipe_file_path);

    // 1. Create the template and register functions
    let mut doc_template = Template::default();
    doc_template.add_funcs(&[
        ("getSource", get_source_fn as fn(&[Value]) -> Result<Value, FuncError>),
        ("sumTimers", sum_timers as fn(&[Value]) -> Result<Value, FuncError>),
        ("lower", lower as fn(&[Value]) -> Result<Value, FuncError>),
        ("replace", replace as fn(&[Value]) -> Result<Value, FuncError>),
        ("round", round as fn(&[Value]) -> Result<Value, FuncError>),
        ("add1", add1 as fn(&[Value]) -> Result<Value, FuncError>),
        ("wrap", wrap as fn(&[Value]) -> Result<Value, FuncError>),
        ("indent", indent as fn(&[Value]) -> Result<Value, FuncError>),
    ]);

    // 2. Load and parse template files
    let templates_list = match get_documentation_templates(
        &recipe.config.recipe_search_root,
        &recipe.info.recipe_file_path,
        &recipe.config.template_files,
    ) {
        Ok(lst) => lst,
        Err(e) => {
            log::warn!("Error getting template data {}: {}", recipe.info.recipe_file_path, e);
            return;
        }
    };

    for t in templates_list {
        if let Err(e) = doc_template.parse(t) {
            log::warn!("Error parsing template {}: {}", recipe.info.recipe_file_path, e);
            return;
        }
    }

    // 3. Serialize context to json value and then map to gtmpl value
    let json_val = match serde_json::to_value(recipe) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Error serializing recipe to JSON for {}: {}", recipe.info.recipe_file_path, e);
            return;
        }
    };
    let gtmpl_context_value = json_to_gtmpl(json_val);
    let gtmpl_context = gtmpl_ng::Context::from(gtmpl_context_value);

    // 4. Render template
    let rendered = match doc_template.render(&gtmpl_context) {
        Ok(out) => out,
        Err(e) => {
            log::warn!("Error executing template {}: {}", recipe.info.recipe_file_path, e);
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
            log::warn!("Could not create parent directory for recipe markdown file {}, skipping recipe: {}", recipe.info.new_recipe_file_path, e);
            return;
        }
        if let Err(e) = fs::write(out_path, formatted) {
            log::warn!("Error generating documentation file for recipe {}: {}", recipe.info.new_recipe_file_path, e);
        }
    }
}

use gtmpl_ng::{FuncError, Value, Template};
use std::collections::HashMap;

#[test]
fn test_is_relative_path() {
    assert!(cook_docs::cook::is_relative_path("./recipes"));
    assert!(!cook_docs::cook::is_relative_path("/absolute/path"));
}

#[test]
fn test_is_base_filename() {
    assert!(cook_docs::cook::is_base_filename("recipe.md.gotmpl"));
    assert!(!cook_docs::cook::is_base_filename("path/to/recipe.md.gotmpl"));
}

#[test]
fn test_get_recipe_name() {
    assert_eq!(cook_docs::cook::get_recipe_name("recipes/Tomato Soup.cook"), "Tomato Soup");
}

#[test]
fn test_load_config_defaults() {
    let config = cook_docs::config::load_config(&["cook-docs"]).unwrap();
    assert_eq!(config.dry_run, false);
    assert_eq!(config.jsonify, false);
    assert_eq!(config.ignore_file, ".cookdocsignore");
    assert_eq!(config.recipe_search_root, ".");
    assert_eq!(config.log_level, "info");
    assert_eq!(config.template_files, vec!["recipe.md.gotmpl"]);
    assert_eq!(config.word_wrap, 120);
    assert_eq!(config.output_dir, "");
}

#[test]
fn test_load_config_cli_overrides() {
    let config = cook_docs::config::load_config(&[
        "cook-docs",
        "-d",
        "-j",
        "-i",
        "custom-ignore",
        "-c",
        "custom-root",
        "-l",
        "debug",
        "-t",
        "t1.gotmpl,t2.gotmpl",
        "-w",
        "80",
        "-o",
        "out",
    ])
    .unwrap();
    assert_eq!(config.dry_run, true);
    assert_eq!(config.jsonify, true);
    assert_eq!(config.ignore_file, "custom-ignore");
    assert_eq!(config.recipe_search_root, "custom-root");
    assert_eq!(config.log_level, "debug");
    assert_eq!(config.template_files, vec!["t1.gotmpl", "t2.gotmpl"]);
    assert_eq!(config.word_wrap, 80);
    assert_eq!(config.output_dir, "out");
}

#[test]
fn test_metadata_keys() {
    let parser = cooklang::CooklangParser::new(cooklang::Extensions::all(), cooklang::Converter::default());
    let recipe = parser.parse(">> key: value\n").into_output().unwrap();
    let mut metadata_map = HashMap::new();
    for (k, v) in &recipe.metadata.map {
        let key_str = if let Some(s) = k.as_str() {
            s.to_string()
        } else {
            serde_json::to_string(k).unwrap_or_default()
        };
        let val_str = if let Some(s) = v.as_str() {
            s.to_string()
        } else if let Some(b) = v.as_bool() {
            b.to_string()
        } else if let Some(n) = v.as_f64() {
            n.to_string()
        } else if let Some(n) = v.as_i64() {
            n.to_string()
        } else {
            serde_json::to_string(v).unwrap_or_default()
        };
        metadata_map.insert(key_str, val_str);
    }
    assert_eq!(metadata_map.get("key"), Some(&"value".to_string()));
}

#[test]
fn test_cooklang_types() {
    let parser = cooklang::CooklangParser::new(cooklang::Extensions::all(), cooklang::Converter::default());
    let recipe = parser.parse("Add @flour{1-2%g}").into_output().unwrap();
    let ing = &recipe.ingredients[0];
    let qty = ing.quantity.as_ref().unwrap();
    match qty.value() {
        cooklang::quantity::Value::Number(num) => {
            match num {
                cooklang::quantity::Number::Regular(f) => {
                    println!("Regular: {}", f);
                }
                cooklang::quantity::Number::Fraction { whole, num, den, .. } => {
                    println!("Fraction: {} {}/{}", whole, num, den);
                }
            }
        }
        cooklang::quantity::Value::Range { start, end } => {
            println!("Range: {:?} to {:?}", start, end);
        }
        cooklang::quantity::Value::Text(t) => {
            println!("Text: {}", t);
        }
    }
}

fn dummy_fn(args: &[Value]) -> Result<Value, FuncError> {
    Ok(args.first().cloned().unwrap_or(Value::Nil))
}

#[test]
fn test_template_funcs() {
    let mut t = Template::default();
    t.add_funcs(&[
        ("dummy", dummy_fn as fn(&[Value]) -> Result<Value, FuncError>)
    ]);
}

#[test]
fn test_gtmpl_number_getters() {
    let n2: Value = 820i64.into();
    if let Value::Number(num) = n2 {
        assert_eq!(num.as_i64(), Some(820));
        assert_eq!(num.as_u64(), Some(820));
    }
}

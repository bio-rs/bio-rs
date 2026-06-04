use wasm_bindgen_test::*;

const WORKFLOW_OUTPUT_SCHEMA: &str =
    include_str!("../../../../schemas/sequence-workflow-output.v0.json");

#[wasm_bindgen_test]
fn test_parse_fasta() {
    let fasta = ">seq1\nACDE\n>seq2\nFGHI\n";
    let bytes = fasta.as_bytes();
    let result = biors_wasm::parse_fasta(bytes);
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_validate_fasta_protein() {
    let fasta = ">seq1\nACDE\n";
    let bytes = fasta.as_bytes();
    let result = biors_wasm::validate_fasta(bytes, "protein".to_string());
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_validate_fasta_auto() {
    let fasta = ">seq1\nACDE\n";
    let bytes = fasta.as_bytes();
    let result = biors_wasm::validate_fasta(bytes, "auto".to_string());
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_validate_fasta_invalid_kind() {
    let fasta = ">seq1\nACDE\n";
    let bytes = fasta.as_bytes();
    let result = biors_wasm::validate_fasta(bytes, "invalid".to_string());
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_tokenize() {
    let records = js_sys::JSON::parse(r#"[{"id":"seq1","sequence":"ACDE"}]"#).unwrap();
    let result = biors_wasm::tokenize(records, "protein-20".to_string());
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_tokenize_accepts_nucleotide_profiles() {
    let records = js_sys::JSON::parse(r#"[{"id":"dna","sequence":"ACGT"}]"#).unwrap();
    let result = biors_wasm::tokenize(records, "dna-iupac".to_string()).unwrap();
    let records = js_sys::Reflect::get(&result, &"records".into()).unwrap();
    let first_record = js_sys::Array::from(&records).get(0);
    let alphabet = js_sys::Reflect::get(&first_record, &"alphabet".into())
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(alphabet, "dna-iupac");
}

#[wasm_bindgen_test]
fn test_tokenize_invalid_profile() {
    let records = js_sys::JSON::parse(r#"[{"id":"seq1","sequence":"ACDE"}]"#).unwrap();
    let result = biors_wasm::tokenize(records, "invalid".to_string());
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_build_model_input() {
    let tokenized = js_sys::JSON::parse(r#"[{"id":"seq1","tokens":[0,1,2,3],"length":4,"alphabet":"protein-20","valid":true,"warnings":[],"errors":[]}]"#).unwrap();
    let result = biors_wasm::build_model_input(tokenized, 8);
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_build_model_input_with_policy() {
    let tokenized = js_sys::JSON::parse(r#"[{"id":"seq1","tokens":[0,1,2,3],"length":4,"alphabet":"protein-20","valid":true,"warnings":[],"errors":[]}]"#).unwrap();
    let result =
        biors_wasm::build_model_input_with_policy(tokenized, 8, 21, "fixed_length".to_string());
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_build_model_input_rejects_empty_token_sequence() {
    let tokenized = js_sys::JSON::parse(
        r#"[{"id":"empty","tokens":[],"length":0,"alphabet":"protein-20","valid":true,"warnings":[],"errors":[]}]"#,
    )
    .unwrap();
    let error = biors_wasm::build_model_input(tokenized, 8).expect_err("empty tokens fail");
    assert!(error.as_string().unwrap().contains("empty"));
}

#[wasm_bindgen_test]
fn test_run_workflow() {
    let config = workflow_config(">seq1\nACDE\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"padding".into(), &"fixed_length".into()).unwrap();
    js_sys::Reflect::set(&config, &"padTokenId".into(), &0.into()).unwrap();
    js_sys::Reflect::set(&config, &"kind".into(), &"protein".into()).unwrap();
    js_sys::Reflect::set(&config, &"profile".into(), &"protein-20".into()).unwrap();

    let result = biors_wasm::run_workflow(config.into());
    let output = result.unwrap();
    let model_ready = js_sys::Reflect::get(&output, &"model_ready".into())
        .unwrap()
        .as_bool()
        .unwrap();
    assert!(model_ready);
    let model_input = js_sys::Reflect::get(&output, &"model_input".into()).unwrap();
    let records = js_sys::Reflect::get(&model_input, &"records".into()).unwrap();
    let first_record = js_sys::Array::from(&records).get(0);
    let input_ids =
        js_sys::Array::from(&js_sys::Reflect::get(&first_record, &"input_ids".into()).unwrap());
    assert_eq!(input_ids.length(), 8);

    let provenance = js_sys::Reflect::get(&output, &"provenance".into()).unwrap();
    let input_hash = js_sys::Reflect::get(&provenance, &"input_hash".into())
        .unwrap()
        .as_string()
        .unwrap();
    assert!(input_hash.starts_with("fnv1a64:"));
    let tokenizer = js_sys::Reflect::get(&provenance, &"tokenizer".into()).unwrap();
    let tokenizer_name = js_sys::Reflect::get(&tokenizer, &"name".into())
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(tokenizer_name, "protein-20");

    assert_matches_shared_workflow_schema_contract(&output, WORKFLOW_OUTPUT_SCHEMA);
}

#[wasm_bindgen_test]
fn test_run_workflow_accepts_nucleotide_kind_and_profile() {
    let config = workflow_config(">seq1\nACGT\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &6.into()).unwrap();
    js_sys::Reflect::set(&config, &"kind".into(), &"dna".into()).unwrap();
    js_sys::Reflect::set(&config, &"profile".into(), &"dna-iupac".into()).unwrap();
    js_sys::Reflect::set(&config, &"padding".into(), &"fixed_length".into()).unwrap();
    let output = biors_wasm::run_workflow(config.into()).unwrap();
    let workflow = js_sys::Reflect::get(&output, &"workflow".into())
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(workflow, "sequence_model_input.v0");
    let provenance = js_sys::Reflect::get(&output, &"provenance".into()).unwrap();
    let tokenizer = js_sys::Reflect::get(&provenance, &"tokenizer".into()).unwrap();
    let tokenizer_name = js_sys::Reflect::get(&tokenizer, &"name".into())
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(tokenizer_name, "dna-iupac");

    let config = workflow_config(">seq1\nACGT\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"kind".into(), &"auto".into()).unwrap();
    let output = biors_wasm::run_workflow(config.into()).unwrap();
    let provenance = js_sys::Reflect::get(&output, &"provenance".into()).unwrap();
    let tokenizer = js_sys::Reflect::get(&provenance, &"tokenizer".into()).unwrap();
    let tokenizer_name = js_sys::Reflect::get(&tokenizer, &"name".into())
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(tokenizer_name, "dna-iupac");
}

#[wasm_bindgen_test]
fn test_run_workflow_rejects_kind_profile_mismatch() {
    let config = workflow_config(">seq1\nACGT\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"kind".into(), &"dna".into()).unwrap();
    js_sys::Reflect::set(&config, &"profile".into(), &"protein-20".into()).unwrap();
    assert!(biors_wasm::run_workflow(config.into()).is_err());
}

#[wasm_bindgen_test]
fn test_run_workflow_accepts_special_profiles() {
    let config = workflow_config(">seq1\nACDE\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"profile".into(), &"protein-20-special".into()).unwrap();
    assert!(biors_wasm::run_workflow(config.into()).is_ok());
}

#[wasm_bindgen_test]
fn test_run_workflow_rejects_fractional_numeric_config() {
    let config = workflow_config(">seq1\nACDE\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.5.into()).unwrap();

    assert!(biors_wasm::run_workflow(config.into()).is_err());

    let config = workflow_config(">seq1\nACDE\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"padTokenId".into(), &21.5.into()).unwrap();

    assert!(biors_wasm::run_workflow(config.into()).is_err());
}

#[wasm_bindgen_test]
fn test_run_workflow_accepts_missing_pad_token_id() {
    let config = workflow_config(">seq1\nACDE\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"padding".into(), &"fixed_length".into()).unwrap();

    let result = biors_wasm::run_workflow(config.into());
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_run_workflow_rejects_invalid_pad_token_id_values() {
    assert_pad_token_id_rejected(&"21".into());
    assert_pad_token_id_rejected(&(-1).into());
    assert_pad_token_id_rejected(&256.into());
}

fn assert_pad_token_id_rejected(value: &wasm_bindgen::JsValue) {
    let config = workflow_config(">seq1\nACDE\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"padTokenId".into(), value).unwrap();

    let error = biors_wasm::run_workflow(config.into()).expect_err("padTokenId should fail");
    assert!(error
        .as_string()
        .expect("error message")
        .contains("field padTokenId must be an integer between 0 and 255"));
}

fn workflow_config(fasta: &str) -> js_sys::Object {
    let config = js_sys::Object::new();
    js_sys::Reflect::set(
        &config,
        &"fastaBytes".into(),
        &js_sys::Uint8Array::from(fasta.as_bytes()).into(),
    )
    .unwrap();
    config
}

fn assert_matches_shared_workflow_schema_contract(
    output: &wasm_bindgen::JsValue,
    schema_json: &str,
) {
    let output_json = js_sys::JSON::stringify(output)
        .unwrap()
        .as_string()
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output_json).unwrap();
    let schema: serde_json::Value = serde_json::from_str(schema_json).unwrap();

    validate_json_schema_subset(&value, &schema, &schema, "$").expect("WASM workflow schema match");
}

fn validate_json_schema_subset(
    value: &serde_json::Value,
    schema: &serde_json::Value,
    root: &serde_json::Value,
    path: &str,
) -> Result<(), String> {
    let schema = resolve_schema_ref(schema, root)?;

    if let Some(options) = schema.get("anyOf").and_then(serde_json::Value::as_array) {
        if options
            .iter()
            .any(|option| validate_json_schema_subset(value, option, root, path).is_ok())
        {
            return Ok(());
        }
        return Err(format!("{path} did not match any anyOf schema"));
    }

    if let Some(expected) = schema.get("const") {
        if value != expected {
            return Err(format!("{path} expected const {expected}, got {value}"));
        }
    }
    if let Some(values) = schema.get("enum").and_then(serde_json::Value::as_array) {
        if !values.contains(value) {
            return Err(format!("{path} value {value} is outside enum {values:?}"));
        }
    }

    if let Some(type_name) = schema.get("type").and_then(serde_json::Value::as_str) {
        validate_type(value, type_name, path)?;
    }

    if let Some(minimum) = schema.get("minimum").and_then(serde_json::Value::as_i64) {
        let Some(actual) = value.as_i64() else {
            return Err(format!("{path} must be an integer for minimum check"));
        };
        if actual < minimum {
            return Err(format!("{path} value {actual} is below minimum {minimum}"));
        }
    }
    if let Some(maximum) = schema.get("maximum").and_then(serde_json::Value::as_i64) {
        let Some(actual) = value.as_i64() else {
            return Err(format!("{path} must be an integer for maximum check"));
        };
        if actual > maximum {
            return Err(format!("{path} value {actual} is above maximum {maximum}"));
        }
    }
    if let Some(min_length) = schema.get("minLength").and_then(serde_json::Value::as_u64) {
        let Some(actual) = value.as_str() else {
            return Err(format!("{path} must be a string for minLength check"));
        };
        if actual.chars().count() < min_length as usize {
            return Err(format!("{path} is shorter than minLength {min_length}"));
        }
    }
    if let Some(max_length) = schema.get("maxLength").and_then(serde_json::Value::as_u64) {
        let Some(actual) = value.as_str() else {
            return Err(format!("{path} must be a string for maxLength check"));
        };
        if actual.chars().count() > max_length as usize {
            return Err(format!("{path} is longer than maxLength {max_length}"));
        }
    }
    if let Some(pattern) = schema.get("pattern").and_then(serde_json::Value::as_str) {
        validate_known_pattern(value, pattern, path)?;
    }

    if let Some(object) = value.as_object() {
        validate_object_schema(object, &schema, root, path)?;
    }
    if let Some(array) = value.as_array() {
        validate_array_schema(array, &schema, root, path)?;
    }

    Ok(())
}

fn resolve_schema_ref<'a>(
    schema: &'a serde_json::Value,
    root: &'a serde_json::Value,
) -> Result<&'a serde_json::Value, String> {
    let Some(reference) = schema.get("$ref").and_then(serde_json::Value::as_str) else {
        return Ok(schema);
    };
    let Some(name) = reference.strip_prefix("#/$defs/") else {
        return Err(format!("unsupported schema ref {reference}"));
    };
    root.pointer(&format!("/$defs/{name}"))
        .ok_or_else(|| format!("missing schema ref {reference}"))
}

fn validate_object_schema(
    object: &serde_json::Map<String, serde_json::Value>,
    schema: &serde_json::Value,
    root: &serde_json::Value,
    path: &str,
) -> Result<(), String> {
    let properties = schema
        .get("properties")
        .and_then(serde_json::Value::as_object);
    if let Some(required) = schema.get("required").and_then(serde_json::Value::as_array) {
        for key in required {
            let key = key
                .as_str()
                .ok_or_else(|| format!("{path} required key is not string"))?;
            if !object.contains_key(key) {
                return Err(format!("{path} missing required key {key}"));
            }
        }
    }
    if schema
        .get("additionalProperties")
        .and_then(serde_json::Value::as_bool)
        == Some(false)
    {
        let Some(properties) = properties else {
            if object.is_empty() {
                return Ok(());
            }
            return Err(format!("{path} disallows all additional properties"));
        };
        for key in object.keys() {
            if !properties.contains_key(key) {
                return Err(format!("{path} has non-schema key {key}"));
            }
        }
    }
    if let Some(properties) = properties {
        for (key, child_schema) in properties {
            if let Some(child_value) = object.get(key) {
                validate_json_schema_subset(
                    child_value,
                    child_schema,
                    root,
                    &format!("{path}.{key}"),
                )?;
            }
        }
    }
    Ok(())
}

fn validate_array_schema(
    array: &[serde_json::Value],
    schema: &serde_json::Value,
    root: &serde_json::Value,
    path: &str,
) -> Result<(), String> {
    if let Some(min_items) = schema.get("minItems").and_then(serde_json::Value::as_u64) {
        if array.len() < min_items as usize {
            return Err(format!("{path} has fewer than minItems {min_items}"));
        }
    }
    if let Some(item_schema) = schema.get("items") {
        for (index, item) in array.iter().enumerate() {
            validate_json_schema_subset(item, item_schema, root, &format!("{path}[{index}]"))?;
        }
    }
    Ok(())
}

fn validate_type(value: &serde_json::Value, type_name: &str, path: &str) -> Result<(), String> {
    let valid = match type_name {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        other => return Err(format!("{path} uses unsupported schema type {other}")),
    };
    if valid {
        Ok(())
    } else {
        Err(format!("{path} expected type {type_name}, got {value}"))
    }
}

fn validate_known_pattern(
    value: &serde_json::Value,
    pattern: &str,
    path: &str,
) -> Result<(), String> {
    let Some(actual) = value.as_str() else {
        return Err(format!("{path} must be a string for pattern check"));
    };
    match pattern {
        "^sha256:[0-9a-f]{64}$" => validate_prefixed_hex(actual, "sha256:", 64, path),
        "^fnv1a64:[0-9a-f]{16}$" => validate_prefixed_hex(actual, "fnv1a64:", 16, path),
        _ => Ok(()),
    }
}

fn validate_prefixed_hex(
    value: &str,
    prefix: &str,
    hex_len: usize,
    path: &str,
) -> Result<(), String> {
    let Some(hex) = value.strip_prefix(prefix) else {
        return Err(format!("{path} value {value} is missing prefix {prefix}"));
    };
    if hex.len() != hex_len
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(format!(
            "{path} value {value} does not match lowercase hex length {hex_len}"
        ));
    }
    Ok(())
}

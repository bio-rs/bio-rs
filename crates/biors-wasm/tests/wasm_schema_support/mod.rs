pub fn assert_matches_shared_workflow_schema_contract(
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
        validate_object_schema(object, schema, root, path)?;
    }
    if let Some(array) = value.as_array() {
        validate_array_schema(array, schema, root, path)?;
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

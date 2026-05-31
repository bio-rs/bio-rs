use biors_core::model_input::{ModelInputPolicy, PaddingPolicy};
use biors_core::workflow::prepare_protein_model_input_workflow;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = runWorkflow)]
pub fn run_workflow(config: JsValue) -> Result<JsValue, JsValue> {
    let fasta_bytes = get_bytes(&config, "fastaBytes")?;
    let max_length = get_usize(&config, "maxLength")?;
    let pad_token_id = get_u8_opt(&config, "padTokenId").unwrap_or(0);
    let padding = get_string_opt(&config, "padding");

    let fasta_text =
        std::str::from_utf8(&fasta_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let records = biors_core::fasta::parse_fasta_records(fasta_text)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let padding_policy = match padding.as_deref() {
        Some("no_padding") | None => PaddingPolicy::NoPadding,
        Some("fixed_length") => PaddingPolicy::FixedLength,
        Some(other) => {
            return Err(JsValue::from_str(&format!(
                "invalid padding: '{}'. Expected 'fixed_length' or 'no_padding'",
                other
            )))
        }
    };

    let policy = ModelInputPolicy {
        max_length,
        pad_token_id,
        padding: padding_policy,
    };

    let input_hash = biors_core::hash::sha256_bytes_digest(&fasta_bytes);

    let output = prepare_protein_model_input_workflow(input_hash, &records, policy)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let json = serde_json::to_string(&output).map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_sys::JSON::parse(&json)
}

fn get_bytes(obj: &JsValue, key: &str) -> Result<Vec<u8>, JsValue> {
    let val = js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .map_err(|_| JsValue::from_str(&format!("missing field: {}", key)))?;
    let arr = val
        .dyn_into::<js_sys::Uint8Array>()
        .map_err(|_| JsValue::from_str(&format!("field {} must be a Uint8Array", key)))?;
    Ok(arr.to_vec())
}

fn get_usize(obj: &JsValue, key: &str) -> Result<usize, JsValue> {
    let val = js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .map_err(|_| JsValue::from_str(&format!("missing field: {}", key)))?;
    val.as_f64()
        .and_then(|f| if f >= 0.0 { Some(f as usize) } else { None })
        .ok_or_else(|| JsValue::from_str(&format!("field {} must be a non-negative integer", key)))
}

fn get_u8_opt(obj: &JsValue, key: &str) -> Option<u8> {
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_f64())
        .and_then(|f| {
            if (0.0..=255.0).contains(&f) {
                Some(f as u8)
            } else {
                None
            }
        })
}

fn get_string_opt(obj: &JsValue, key: &str) -> Option<String> {
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_string())
}

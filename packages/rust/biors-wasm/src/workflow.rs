use biors_core::model_input::{ModelInputPolicy, PaddingPolicy};
use biors_core::sequence::{SequenceKind, SequenceKindSelection};
use biors_core::workflow::prepare_protein_model_input_workflow;
use std::io::Cursor;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = runWorkflow)]
pub fn run_workflow(config: JsValue) -> Result<JsValue, JsValue> {
    let fasta_bytes = get_bytes(&config, "fastaBytes")?;
    let max_length = get_usize(&config, "maxLength")?;
    let pad_token_id = get_u8_opt(&config, "padTokenId")?.unwrap_or(0);
    let padding = get_string_opt(&config, "padding")?;
    let kind = get_string_opt(&config, "kind")?;
    let profile = get_string_opt(&config, "profile")?;

    ensure_supported_workflow_kind(kind.as_deref(), &fasta_bytes)?;
    ensure_default_workflow_profile(profile.as_deref())?;

    let input = biors_core::fasta::parse_fasta_records_reader(Cursor::new(&fasta_bytes))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let padding_policy = match padding.as_deref() {
        Some("no_padding") | None => PaddingPolicy::NoPadding,
        Some("fixed_length") => PaddingPolicy::FixedLength,
        Some(other) => {
            return Err(JsValue::from_str(&format!(
                "invalid padding: '{other}'. Expected 'fixed_length' or 'no_padding'"
            )))
        }
    };

    let policy = ModelInputPolicy {
        max_length,
        pad_token_id,
        padding: padding_policy,
    };

    let output = prepare_protein_model_input_workflow(input.input_hash, &input.records, policy)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let json = serde_json::to_string(&output).map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_sys::JSON::parse(&json)
}

fn ensure_supported_workflow_kind(kind: Option<&str>, fasta_bytes: &[u8]) -> Result<(), JsValue> {
    match kind {
        None | Some("auto") => reject_non_protein_auto_detected_input(fasta_bytes),
        Some("protein") => Ok(()),
        Some("dna") | Some("rna") => Err(JsValue::from_str(
            "unsupported workflow kind: runWorkflow currently supports protein model-input workflows only",
        )),
        Some(other) => Err(JsValue::from_str(&format!(
            "invalid kind: '{other}'. Expected 'auto' or 'protein'"
        ))),
    }
}

fn reject_non_protein_auto_detected_input(fasta_bytes: &[u8]) -> Result<(), JsValue> {
    let fasta_text = std::str::from_utf8(fasta_bytes)
        .map_err(|e| JsValue::from_str(&format!("invalid UTF-8 FASTA input: {e}")))?;
    let report = biors_core::sequence::kind_validation::validate_fasta_input_with_kind(
        fasta_text,
        SequenceKindSelection::Auto,
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
    if report
        .sequences
        .iter()
        .any(|record| record.kind != SequenceKind::Protein)
    {
        return Err(JsValue::from_str(
            "unsupported workflow kind: runWorkflow auto-detected non-protein input, but the workflow currently supports protein model-input workflows only",
        ));
    }
    Ok(())
}

fn ensure_default_workflow_profile(profile: Option<&str>) -> Result<(), JsValue> {
    match profile {
        None | Some("protein-20") => Ok(()),
        Some("protein-20-special") => Err(JsValue::from_str(
            "unsupported workflow profile: runWorkflow currently supports protein-20 only",
        )),
        Some(other) => Err(JsValue::from_str(&format!(
            "invalid profile: '{other}'. Expected 'protein-20'"
        ))),
    }
}

fn get_bytes(obj: &JsValue, key: &str) -> Result<Vec<u8>, JsValue> {
    let val = js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .map_err(|_| JsValue::from_str(&format!("missing field: {key}")))?;
    let arr = val
        .dyn_into::<js_sys::Uint8Array>()
        .map_err(|_| JsValue::from_str(&format!("field {key} must be a Uint8Array")))?;
    Ok(arr.to_vec())
}

fn get_usize(obj: &JsValue, key: &str) -> Result<usize, JsValue> {
    let val = js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .map_err(|_| JsValue::from_str(&format!("missing field: {key}")))?;
    val.as_f64()
        .and_then(|f| {
            if f >= 0.0 && f.fract() == 0.0 {
                Some(f as usize)
            } else {
                None
            }
        })
        .ok_or_else(|| JsValue::from_str(&format!("field {key} must be a non-negative integer")))
}

fn get_u8_opt(obj: &JsValue, key: &str) -> Result<Option<u8>, JsValue> {
    let val = js_sys::Reflect::get(obj, &JsValue::from_str(key)).map_err(|_| {
        JsValue::from_str(&format!("field {key} must be an integer between 0 and 255"))
    })?;
    if val.is_undefined() {
        return Ok(None);
    }
    let Some(number) = val.as_f64() else {
        return Err(JsValue::from_str(&format!(
            "field {key} must be an integer between 0 and 255"
        )));
    };
    if (0.0..=255.0).contains(&number) && number.fract() == 0.0 {
        Ok(Some(number as u8))
    } else {
        Err(JsValue::from_str(&format!(
            "field {key} must be an integer between 0 and 255"
        )))
    }
}

fn get_string_opt(obj: &JsValue, key: &str) -> Result<Option<String>, JsValue> {
    let val = js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .map_err(|_| JsValue::from_str(&format!("field {key} must be a string")))?;
    if val.is_undefined() {
        return Ok(None);
    }
    val.as_string()
        .map(Some)
        .ok_or_else(|| JsValue::from_str(&format!("field {key} must be a string")))
}

use biors_core::sequence::{SequenceKind, SequenceKindSelection};
use serde::Serialize;
use wasm_bindgen::prelude::*;

fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let json = serde_json::to_string(value).map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_sys::JSON::parse(&json)
}

/// Parse FASTA bytes into an array of sequence records.
#[wasm_bindgen(js_name = parseFasta)]
pub fn parse_fasta(bytes: &[u8]) -> Result<JsValue, JsValue> {
    let text = std::str::from_utf8(bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let records = biors_core::fasta::parse_fasta_records(text)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let output: Vec<FastaRecord> = records
        .into_iter()
        .map(|r| FastaRecord {
            id: r.id,
            sequence: String::from_utf8_lossy(&r.sequence).to_string(),
        })
        .collect();
    to_js_value(&output)
}

/// Validate FASTA bytes and return a structured validation report.
#[wasm_bindgen(js_name = validateFasta)]
pub fn validate_fasta(bytes: &[u8], kind: String) -> Result<JsValue, JsValue> {
    let text = std::str::from_utf8(bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let selection = match kind.as_str() {
        "auto" => SequenceKindSelection::Auto,
        "protein" => SequenceKindSelection::Explicit(SequenceKind::Protein),
        "dna" => SequenceKindSelection::Explicit(SequenceKind::Dna),
        "rna" => SequenceKindSelection::Explicit(SequenceKind::Rna),
        _ => {
            return Err(JsValue::from_str(&format!(
                "invalid kind: '{}'. Expected 'auto', 'protein', 'dna', or 'rna'",
                kind
            )))
        }
    };
    let report =
        biors_core::sequence::kind_validation::validate_fasta_input_with_kind(text, selection)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
    to_js_value(&report)
}

#[derive(Serialize)]
struct FastaRecord {
    id: String,
    sequence: String,
}

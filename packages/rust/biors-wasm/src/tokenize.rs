use biors_core::sequence::ProteinSequence;
use biors_core::tokenizer::{tokenize_protein_with_config, TokenizedProtein};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = tokenize)]
pub fn tokenize(records: JsValue, profile: String) -> Result<JsValue, JsValue> {
    let json_str = js_sys::JSON::stringify(&records)?
        .as_string()
        .ok_or_else(|| JsValue::from_str("failed to stringify records"))?;
    let records: Vec<FastaRecord> =
        serde_json::from_str(&json_str).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let config = crate::profile::tokenizer_config_for_profile(&profile)?;

    let mut tokenized: Vec<TokenizedProtein> = Vec::with_capacity(records.len());
    let mut input_ids: Vec<Vec<u8>> = Vec::with_capacity(records.len());
    let mut attention_masks: Vec<Vec<u8>> = Vec::with_capacity(records.len());
    let mut ids: Vec<String> = Vec::with_capacity(records.len());

    for record in records {
        let protein = ProteinSequence {
            id: record.id.clone(),
            sequence: record.sequence.into_bytes(),
        };
        let t = tokenize_protein_with_config(&protein, &config);
        input_ids.push(t.tokens.clone());
        attention_masks.push(vec![1u8; t.tokens.len()]);
        ids.push(record.id);
        tokenized.push(t);
    }

    let output = TokenizeOutput {
        input_ids,
        attention_mask: attention_masks,
        ids,
        records: tokenized,
    };

    let json = serde_json::to_string(&output).map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_sys::JSON::parse(&json)
}

#[derive(Deserialize, Serialize)]
struct FastaRecord {
    id: String,
    sequence: String,
}

#[derive(Serialize)]
struct TokenizeOutput {
    #[serde(rename = "inputIds")]
    input_ids: Vec<Vec<u8>>,
    #[serde(rename = "attentionMask")]
    attention_mask: Vec<Vec<u8>>,
    ids: Vec<String>,
    records: Vec<TokenizedProtein>,
}

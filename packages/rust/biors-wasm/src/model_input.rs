use biors_core::model_input::{build_model_inputs_checked, ModelInputPolicy, PaddingPolicy};
use biors_core::tokenizer::TokenizedProtein;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = buildModelInput)]
pub fn build_model_input(tokenized: JsValue, max_length: usize) -> Result<JsValue, JsValue> {
    let json_str = js_sys::JSON::stringify(&tokenized)?
        .as_string()
        .ok_or_else(|| JsValue::from_str("failed to stringify tokenized"))?;
    let records: Vec<TokenizedProtein> =
        serde_json::from_str(&json_str).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let policy = ModelInputPolicy {
        max_length,
        pad_token_id: 0,
        padding: PaddingPolicy::NoPadding,
    };

    let model_input = build_model_inputs_checked(&records, policy)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let json =
        serde_json::to_string(&model_input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_sys::JSON::parse(&json)
}

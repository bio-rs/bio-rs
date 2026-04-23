use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn tokenize(input: &str) -> Result<JsValue, JsError> {
    let result = biors_core::tokenize_fasta(input).map_err(|e| JsError::new(&e.to_string()))?;

    Ok(serde_wasm_bindgen::to_value(&result)?)
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

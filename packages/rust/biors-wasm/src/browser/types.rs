use biors_core::tokenizer::TokenizedProtein;
use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::prelude::*;

pub(super) const BROWSER_TOOLING_SCHEMA_VERSION: &str = "biors.browser_tooling.v0";
pub(super) const MAX_BROWSER_INPUT_BYTES: usize = 64 * 1024 * 1024;
pub(super) const WARN_BROWSER_INPUT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Serialize)]
pub(super) struct BrowserExecutionPolicy {
    pub(super) schema_version: &'static str,
    pub(super) execution_mode: &'static str,
    pub(super) network_access: &'static str,
    pub(super) uploads_input_data: bool,
    pub(super) external_model_calls: bool,
    pub(super) persistence: &'static str,
    pub(super) max_input_bytes: usize,
    pub(super) warning_input_bytes: usize,
    pub(super) streaming: BrowserStreamingPolicy,
    pub(super) supported_validation_formats: &'static [&'static str],
    pub(super) supported_tokenization_formats: &'static [&'static str],
}

#[derive(Serialize)]
pub(super) struct BrowserStreamingPolicy {
    pub(super) supported: bool,
    pub(super) behavior: &'static str,
    pub(super) caller_guidance: &'static str,
}

#[derive(Serialize)]
pub(super) struct BrowserFileInspection {
    pub(super) schema_version: &'static str,
    pub(super) file: BrowserFileDescriptor,
    pub(super) accepted: bool,
    pub(super) warnings: Vec<BrowserFileWarning>,
}

#[derive(Serialize)]
pub(super) struct BrowserValidationOutput {
    pub(super) schema_version: &'static str,
    pub(super) file: BrowserFileDescriptor,
    pub(super) report: Value,
    pub(super) warnings: Vec<BrowserFileWarning>,
}

#[derive(Serialize)]
pub(super) struct BrowserTokenizationOutput {
    pub(super) schema_version: &'static str,
    pub(super) file: BrowserFileDescriptor,
    pub(super) tokenization: BrowserTokenizeRecords,
    pub(super) model_input_policy_hint: ModelInputPolicyHint,
    pub(super) warnings: Vec<BrowserFileWarning>,
}

#[derive(Serialize)]
pub(super) struct BrowserFileDescriptor {
    pub(super) name: Option<String>,
    pub(super) format: &'static str,
    pub(super) size_bytes: usize,
    pub(super) content_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) input_hash: Option<String>,
}

#[derive(Serialize)]
pub(super) struct BrowserFileWarning {
    pub(super) code: &'static str,
    pub(super) message: &'static str,
}

#[derive(Serialize)]
pub(super) struct BrowserTokenizeRecords {
    #[serde(rename = "inputIds")]
    pub(super) input_ids: Vec<Vec<u8>>,
    #[serde(rename = "attentionMask")]
    pub(super) attention_mask: Vec<Vec<u8>>,
    pub(super) ids: Vec<String>,
    pub(super) records: Vec<TokenizedProtein>,
}

#[derive(Serialize)]
pub(super) struct ModelInputPolicyHint {
    pub(super) max_length_required: bool,
    pub(super) supported_padding: &'static [&'static str],
    pub(super) note: &'static str,
}

pub(super) fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let json = serde_json::to_string(value).map_err(json_error)?;
    js_sys::JSON::parse(&json)
}

pub(super) fn json_error(error: serde_json::Error) -> JsValue {
    JsValue::from_str(&error.to_string())
}

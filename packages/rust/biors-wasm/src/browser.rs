mod input;
mod types;

use biors_core::molecule::validate_smiles_reader_with_hash;
use biors_core::structure::validate_pdb_reader_with_hash;
use biors_core::tokenizer::{tokenize_protein_with_config, TokenizedProtein};
use input::{sequence_kind_selection, BrowserFileFormat, BrowserFileInput};
use std::io::Cursor;
use types::{
    json_error, to_js_value, BrowserExecutionPolicy, BrowserFileInspection, BrowserStreamingPolicy,
    BrowserTokenizationOutput, BrowserTokenizeRecords, BrowserValidationOutput,
    ModelInputPolicyHint, BROWSER_TOOLING_SCHEMA_VERSION, MAX_BROWSER_INPUT_BYTES,
    WARN_BROWSER_INPUT_BYTES,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = browserExecutionPolicy)]
pub fn browser_execution_policy() -> Result<JsValue, JsValue> {
    to_js_value(&BrowserExecutionPolicy {
        schema_version: BROWSER_TOOLING_SCHEMA_VERSION,
        execution_mode: "wasm_local",
        network_access: "none",
        uploads_input_data: false,
        external_model_calls: false,
        persistence: "caller_controlled",
        max_input_bytes: MAX_BROWSER_INPUT_BYTES,
        warning_input_bytes: WARN_BROWSER_INPUT_BYTES,
        streaming: BrowserStreamingPolicy {
            supported: false,
            behavior: "single Uint8Array input is validated before parsing",
            caller_guidance: "slice or reject larger files before passing them to WASM",
        },
        supported_validation_formats: &["fasta", "fastq", "pdb", "smiles"],
        supported_tokenization_formats: &["fasta"],
    })
}

#[wasm_bindgen(js_name = inspectBrowserFile)]
pub fn inspect_browser_file(input: JsValue) -> Result<JsValue, JsValue> {
    let file = BrowserFileInput::from_js(&input)?;
    file.ensure_size()?;
    to_js_value(&BrowserFileInspection {
        schema_version: BROWSER_TOOLING_SCHEMA_VERSION,
        file: file.descriptor(None),
        accepted: true,
        warnings: file.memory_warnings(),
    })
}

#[wasm_bindgen(js_name = validateBrowserFile)]
pub fn validate_browser_file(input: JsValue) -> Result<JsValue, JsValue> {
    let file = BrowserFileInput::from_js(&input)?;
    file.ensure_size()?;

    let (input_hash, report) = match file.format {
        BrowserFileFormat::Fasta => {
            let selection = sequence_kind_selection(file.kind.as_deref())?;
            let output = biors_core::sequence::validate_fasta_reader_with_kind_and_hash(
                Cursor::new(&file.bytes),
                selection,
            )
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
            (
                output.input_hash,
                serde_json::to_value(output.report).map_err(json_error)?,
            )
        }
        BrowserFileFormat::Fastq => {
            let output =
                biors_core::formats::validate_fastq_reader_with_hash(Cursor::new(&file.bytes))
                    .map_err(|error| JsValue::from_str(&error.to_string()))?;
            (
                output.input_hash,
                serde_json::to_value(output.report).map_err(json_error)?,
            )
        }
        BrowserFileFormat::Pdb => {
            let output = validate_pdb_reader_with_hash(Cursor::new(&file.bytes))
                .map_err(|error| JsValue::from_str(&error.to_string()))?;
            (
                output.input_hash,
                serde_json::to_value(output.report).map_err(json_error)?,
            )
        }
        BrowserFileFormat::Smiles => {
            let output = validate_smiles_reader_with_hash(Cursor::new(&file.bytes))
                .map_err(|error| JsValue::from_str(&error.to_string()))?;
            (
                output.input_hash,
                serde_json::to_value(output.report).map_err(json_error)?,
            )
        }
    };

    to_js_value(&BrowserValidationOutput {
        schema_version: BROWSER_TOOLING_SCHEMA_VERSION,
        file: file.descriptor(Some(input_hash)),
        report,
        warnings: file.memory_warnings(),
    })
}

#[wasm_bindgen(js_name = tokenizeBrowserFile)]
pub fn tokenize_browser_file(input: JsValue) -> Result<JsValue, JsValue> {
    let file = BrowserFileInput::from_js(&input)?;
    file.ensure_size()?;
    if file.format != BrowserFileFormat::Fasta {
        return Err(JsValue::from_str(
            "tokenizeBrowserFile currently supports only FASTA input",
        ));
    }

    let parsed = biors_core::fasta::parse_fasta_records_reader(Cursor::new(&file.bytes))
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    let profile = file.profile.as_deref().unwrap_or("protein-20");
    let config = crate::profile::tokenizer_config_for_profile(profile)?;
    let tokenization = tokenize_records(parsed.records, &config);

    to_js_value(&BrowserTokenizationOutput {
        schema_version: BROWSER_TOOLING_SCHEMA_VERSION,
        file: file.descriptor(Some(parsed.input_hash)),
        tokenization,
        model_input_policy_hint: ModelInputPolicyHint {
            max_length_required: true,
            supported_padding: &["fixed_length", "no_padding"],
            note: "pass tokenization.records to buildModelInputWithPolicy",
        },
        warnings: file.memory_warnings(),
    })
}

fn tokenize_records(
    records: Vec<biors_core::sequence::ProteinSequence>,
    config: &biors_core::tokenizer::ProteinTokenizerConfig,
) -> BrowserTokenizeRecords {
    let mut tokenized: Vec<TokenizedProtein> = Vec::with_capacity(records.len());
    let mut input_ids = Vec::with_capacity(records.len());
    let mut attention_mask = Vec::with_capacity(records.len());
    let mut ids = Vec::with_capacity(records.len());

    for record in records {
        let tokens = tokenize_protein_with_config(&record, config);
        input_ids.push(tokens.tokens.clone());
        attention_mask.push(vec![1u8; tokens.tokens.len()]);
        ids.push(record.id);
        tokenized.push(tokens);
    }

    BrowserTokenizeRecords {
        input_ids,
        attention_mask,
        ids,
        records: tokenized,
    }
}

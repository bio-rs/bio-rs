use biors_backend_candle::{
    CandleBackend, CandleBackendConfig, CandleDevice, CandleInferenceOutput, CANDLE_ERROR_CODES,
    CANDLE_MODEL_INPUT_FORMAT, CANDLE_OUTPUT_FORMAT,
};
use biors_core::model_input::{ModelInput, ModelInputPolicy, ModelInputRecord, PaddingPolicy};
use biors_core::runtime::{Backend, ExecutionContext};
use candle_core::{safetensors, Device, Tensor};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "biors-candle-{name}-{}-{}.safetensors",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    path
}

fn write_probe_weights(path: &PathBuf) {
    let device = Device::Cpu;
    let tensors = HashMap::from([
        (
            "embedding.weight".to_string(),
            Tensor::new(
                &[[0.0_f32, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
                &device,
            )
            .expect("embedding tensor"),
        ),
        (
            "projection.weight".to_string(),
            Tensor::new(&[[1.0_f32, 0.0], [0.0, 1.0]], &device).expect("projection tensor"),
        ),
        (
            "projection.bias".to_string(),
            Tensor::new(&[0.25_f32, -0.25], &device).expect("bias tensor"),
        ),
    ]);
    safetensors::save(&tensors, path).expect("write safetensors fixture");
}

fn model_input_payload() -> Vec<u8> {
    model_input_payload_with_record(ModelInputRecord {
        id: "protein-a".to_string(),
        input_ids: vec![1, 2, 0, 0],
        attention_mask: vec![1, 1, 0, 0],
        truncated: false,
    })
}

fn model_input_payload_with_record(record: ModelInputRecord) -> Vec<u8> {
    let input = ModelInput {
        policy: ModelInputPolicy {
            max_length: 4,
            pad_token_id: 0,
            padding: PaddingPolicy::FixedLength,
        },
        records: vec![record],
    };
    serde_json::to_vec(&input).expect("model input payload")
}

fn candle_backend(name: &str) -> CandleBackend {
    let weights_path = fixture_path(name);
    write_probe_weights(&weights_path);

    CandleBackend::from_safetensors(CandleBackendConfig {
        backend_id: "candle-linear-probe".to_string(),
        weights_path,
        device: CandleDevice::Cpu,
        embedding_tensor: "embedding.weight".to_string(),
        projection_weight_tensor: "projection.weight".to_string(),
        projection_bias_tensor: Some("projection.bias".to_string()),
        max_input_bytes: Some(16 * 1024),
    })
    .expect("load candle backend")
}

#[test]
fn candle_backend_loads_safetensors_and_scores_model_input_payload() {
    let backend = candle_backend("linear-probe");

    let result = backend
        .execute_checked(ExecutionContext {
            trace_id: Some("trace-candle-001".to_string()),
            input_format: CANDLE_MODEL_INPUT_FORMAT.to_string(),
            requested_output_format: Some(CANDLE_OUTPUT_FORMAT.to_string()),
            payload: model_input_payload(),
            metadata: Vec::new(),
        })
        .expect("execute candle backend");

    assert_eq!(result.trace_id.as_deref(), Some("trace-candle-001"));
    assert_eq!(result.output_format, CANDLE_OUTPUT_FORMAT);
    assert!(result
        .metadata
        .iter()
        .any(|item| item.key == "candle.device" && item.value == "cpu"));

    let output: CandleInferenceOutput =
        serde_json::from_slice(&result.payload).expect("decode candle output");
    assert_eq!(output.records.len(), 1);
    assert_eq!(output.records[0].id, "protein-a");
    assert_eq!(output.records[0].values, vec![0.75, 0.25]);
    assert!(!output.records[0].truncated);
}

#[test]
fn candle_backend_rejects_non_binary_attention_mask() {
    let error = execute_payload_expect_error(
        model_input_payload_with_record(ModelInputRecord {
            id: "protein-a".to_string(),
            input_ids: vec![1, 2, 0, 0],
            attention_mask: vec![1, 2, 0, 0],
            truncated: false,
        }),
        "non-binary-mask",
    );

    assert!(error.contains("model_input.non_binary_attention_mask"));
    assert!(error.contains("attention_mask[1] is 2"));
}

#[test]
fn candle_backend_rejects_attention_mask_length_mismatch() {
    let error = execute_payload_expect_error(
        model_input_payload_with_record(ModelInputRecord {
            id: "protein-a".to_string(),
            input_ids: vec![1, 2],
            attention_mask: vec![1],
            truncated: false,
        }),
        "length-mismatch",
    );

    assert!(error.contains("model_input.length_mismatch"));
    assert!(error.contains("2 input ids but 1 attention-mask values"));
}

#[test]
fn candle_backend_rejects_fixed_length_record_size_mismatch() {
    let error = execute_payload_expect_error(
        model_input_payload_with_record(ModelInputRecord {
            id: "protein-a".to_string(),
            input_ids: vec![1, 2],
            attention_mask: vec![1, 1],
            truncated: false,
        }),
        "fixed-length-size-mismatch",
    );

    assert!(error.contains("model_input.fixed_length_mismatch"));
    assert!(error.contains("expected fixed length 4"));
}

#[test]
fn candle_backend_rejects_empty_unmasked_tokens() {
    let error = execute_payload_expect_error(
        model_input_payload_with_record(ModelInputRecord {
            id: "protein-a".to_string(),
            input_ids: vec![0, 0, 0, 0],
            attention_mask: vec![0, 0, 0, 0],
            truncated: false,
        }),
        "empty-mask",
    );

    assert!(error.contains("model_input.empty_attention_mask"));
    assert!(error.contains("has no unmasked tokens"));
}

#[test]
fn candle_backend_rejects_token_ids_outside_embedding_vocab() {
    let error = execute_payload_expect_error(
        model_input_payload_with_record(ModelInputRecord {
            id: "protein-a".to_string(),
            input_ids: vec![4, 0, 0, 0],
            attention_mask: vec![1, 0, 0, 0],
            truncated: false,
        }),
        "token-out-of-range",
    );

    assert!(error.contains("candle.token_id_out_of_range"));
    assert!(error.contains("token id 4 exceeds embedding vocabulary size 4"));
}

#[test]
fn candle_error_codes_are_registered_in_release_docs() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let error_codes =
        fs::read_to_string(repo.join("docs/error-codes.md")).expect("read error-code registry");
    let candle_docs =
        fs::read_to_string(repo.join("docs/candle-backend.md")).expect("read Candle docs");

    for code in CANDLE_ERROR_CODES {
        assert!(
            error_codes.contains(code),
            "docs/error-codes.md is missing {code}"
        );
        assert!(
            candle_docs.contains(code),
            "docs/candle-backend.md is missing {code}"
        );
    }
}

fn execute_payload_expect_error(payload: Vec<u8>, fixture_name: &str) -> String {
    let backend = candle_backend(fixture_name);
    let error = backend
        .execute_checked(ExecutionContext {
            trace_id: Some(format!("trace-{fixture_name}")),
            input_format: CANDLE_MODEL_INPUT_FORMAT.to_string(),
            requested_output_format: Some(CANDLE_OUTPUT_FORMAT.to_string()),
            payload,
            metadata: Vec::new(),
        })
        .expect_err("payload should be rejected");
    error.to_string()
}

use biors_backend_candle::{
    CandleBackend, CandleBackendConfig, CandleDevice, CandleInferenceOutput,
    CANDLE_MODEL_INPUT_FORMAT, CANDLE_OUTPUT_FORMAT,
};
use biors_core::model_input::{ModelInput, ModelInputPolicy, ModelInputRecord, PaddingPolicy};
use biors_core::runtime::{Backend, ExecutionContext};
use candle_core::{safetensors, Device, Tensor};
use std::collections::HashMap;
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
    let input = ModelInput {
        policy: ModelInputPolicy {
            max_length: 4,
            pad_token_id: 0,
            padding: PaddingPolicy::FixedLength,
        },
        records: vec![ModelInputRecord {
            id: "protein-a".to_string(),
            input_ids: vec![1, 2, 0, 0],
            attention_mask: vec![1, 1, 0, 0],
            truncated: false,
        }],
    };
    serde_json::to_vec(&input).expect("model input payload")
}

#[test]
fn candle_backend_loads_safetensors_and_scores_model_input_payload() {
    let weights_path = fixture_path("linear-probe");
    write_probe_weights(&weights_path);

    let backend = CandleBackend::from_safetensors(CandleBackendConfig {
        backend_id: "candle-linear-probe".to_string(),
        weights_path,
        device: CandleDevice::Cpu,
        embedding_tensor: "embedding.weight".to_string(),
        projection_weight_tensor: "projection.weight".to_string(),
        projection_bias_tensor: Some("projection.bias".to_string()),
        max_input_bytes: Some(16 * 1024),
    })
    .expect("load candle backend");

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

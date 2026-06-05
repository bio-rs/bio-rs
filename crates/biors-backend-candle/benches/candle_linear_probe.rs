use biors_backend_candle::{
    CandleBackend, CandleBackendConfig, CandleDevice, CANDLE_MODEL_INPUT_FORMAT,
    CANDLE_OUTPUT_FORMAT,
};
use biors_core::model_input::{ModelInput, ModelInputPolicy, ModelInputRecord, PaddingPolicy};
use biors_core::runtime::{Backend, ExecutionContext};
use candle_core::{safetensors, Device, Tensor};
use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::path::PathBuf;

fn weights_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "biors-candle-bench-{}.safetensors",
        std::process::id()
    ));
    path
}

fn write_weights(path: &PathBuf) {
    let device = Device::Cpu;
    let vocab = (0..64)
        .flat_map(|token| {
            [
                token as f32 / 64.0,
                (token % 7) as f32 / 7.0,
                (token % 11) as f32 / 11.0,
                1.0,
            ]
        })
        .collect::<Vec<_>>();
    let tensors = HashMap::from([
        (
            "embedding.weight".to_string(),
            Tensor::from_vec(vocab, (64, 4), &device).expect("embedding tensor"),
        ),
        (
            "projection.weight".to_string(),
            Tensor::new(
                &[
                    [0.5_f32, 0.0, 0.1],
                    [0.0, 0.5, 0.1],
                    [0.25, 0.25, 0.2],
                    [0.1, 0.1, 0.3],
                ],
                &device,
            )
            .expect("projection tensor"),
        ),
        (
            "projection.bias".to_string(),
            Tensor::new(&[0.01_f32, -0.01, 0.0], &device).expect("bias tensor"),
        ),
    ]);
    safetensors::save(&tensors, path).expect("write benchmark weights");
}

fn payload() -> Vec<u8> {
    let records = (0..32)
        .map(|index| {
            let input_ids = (0..128)
                .map(|offset| ((index + offset) % 63 + 1) as u8)
                .collect::<Vec<_>>();
            ModelInputRecord {
                id: format!("protein-{index:02}"),
                attention_mask: vec![1; input_ids.len()],
                input_ids,
                truncated: false,
            }
        })
        .collect::<Vec<_>>();
    serde_json::to_vec(&ModelInput {
        policy: ModelInputPolicy {
            max_length: 128,
            pad_token_id: 0,
            padding: PaddingPolicy::NoPadding,
        },
        records,
    })
    .expect("serialize benchmark payload")
}

fn bench_candle_linear_probe(c: &mut Criterion) {
    let path = weights_path();
    write_weights(&path);
    let backend = CandleBackend::from_safetensors(CandleBackendConfig {
        backend_id: "candle-bench-linear-probe".to_string(),
        weights_path: path,
        device: CandleDevice::Cpu,
        embedding_tensor: "embedding.weight".to_string(),
        projection_weight_tensor: "projection.weight".to_string(),
        projection_bias_tensor: Some("projection.bias".to_string()),
        max_input_bytes: Some(1024 * 1024),
    })
    .expect("load benchmark backend");
    let payload = payload();

    c.bench_function("candle_linear_probe_32x128_cpu", |b| {
        b.iter(|| {
            backend
                .execute_checked(ExecutionContext {
                    trace_id: None,
                    input_format: CANDLE_MODEL_INPUT_FORMAT.to_string(),
                    requested_output_format: Some(CANDLE_OUTPUT_FORMAT.to_string()),
                    payload: payload.clone(),
                    metadata: Vec::new(),
                })
                .expect("execute benchmark payload")
        })
    });
}

criterion_group!(benches, bench_candle_linear_probe);
criterion_main!(benches);

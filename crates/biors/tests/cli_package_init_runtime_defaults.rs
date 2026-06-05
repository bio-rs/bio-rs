mod common;
mod package_support;

#[test]
fn package_init_infers_onnx_runtime_defaults_from_extension() {
    let manifest = package_support::run_package_init_with_model("model.onnx");

    assert_eq!(manifest["model"]["format"], "onnx");
    assert_eq!(manifest["model"]["path"], "models/model.onnx");
    assert_eq!(manifest["runtime"]["backend"], "onnx-webgpu");
    assert_eq!(manifest["runtime"]["target"], "browser-wasm-webgpu");
    assert_eq!(manifest["runtime"]["version"], "onnx-webgpu.v0");
}

#[test]
fn package_init_infers_safetensors_runtime_defaults_from_extension() {
    let manifest = package_support::run_package_init_with_model("model.safetensors");

    assert_eq!(manifest["model"]["format"], "safetensors");
    assert_eq!(manifest["model"]["path"], "models/model.safetensors");
    assert_eq!(manifest["runtime"]["backend"], "candle");
    assert_eq!(manifest["runtime"]["target"], "local-cpu");
    assert_eq!(manifest["runtime"]["version"], "candle.v0");
}

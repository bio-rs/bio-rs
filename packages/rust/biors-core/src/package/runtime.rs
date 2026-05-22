use super::{
    validate_package_manifest, BackendCompatibilityCheck, ModelArtifactMetadataSummary,
    ModelFormat, PackageManifest, RuntimeBackend, RuntimeBridgeReport, RuntimeTargetPlatform,
};

/// Build a runtime bridge readiness report from a package manifest.
pub fn plan_runtime_bridge(manifest: &PackageManifest) -> RuntimeBridgeReport {
    let mut blocking_issues = validate_package_manifest(manifest).issues;
    let compatibility_checks = vec![runtime_model_pair_check(manifest)];

    blocking_issues.extend(
        compatibility_checks
            .iter()
            .filter(|check| !check.passed)
            .map(|check| check.message.clone()),
    );

    let backend_id = format!("{}:{}", manifest.name, manifest.runtime.backend);
    let provider = execution_provider(manifest);
    let version = manifest
        .runtime
        .version
        .clone()
        .or_else(|| Some(format!("{}.v0", manifest.runtime.backend)));
    let backend_config = crate::runtime::BackendConfig {
        backend_id,
        provider: provider.clone(),
        version,
        model_artifact: Some(manifest.model.path.clone()),
    };

    RuntimeBridgeReport {
        ready: blocking_issues.is_empty(),
        backend: manifest.runtime.backend,
        target: manifest.runtime.target,
        model_format: manifest.model.format,
        model_metadata: manifest
            .model
            .metadata
            .as_ref()
            .map(ModelArtifactMetadataSummary::from),
        backend_config,
        execution_provider: provider,
        compatibility_checks,
        blocking_issues,
    }
}

fn runtime_model_pair_check(manifest: &PackageManifest) -> BackendCompatibilityCheck {
    let passed = matches!(
        (
            manifest.model.format,
            manifest.runtime.backend,
            manifest.runtime.target
        ),
        (
            ModelFormat::Onnx,
            RuntimeBackend::OnnxWebgpu,
            RuntimeTargetPlatform::BrowserWasmWebgpu
        ) | (
            ModelFormat::Safetensors,
            RuntimeBackend::Candle,
            RuntimeTargetPlatform::LocalCpu
        )
    );

    let message = if passed {
        format!(
            "{}/{} supports {} model artifacts",
            manifest.runtime.backend, manifest.runtime.target, manifest.model.format
        )
    } else {
        format!(
            "model format '{}' is not compatible with backend '{}' target '{}'",
            manifest.model.format, manifest.runtime.backend, manifest.runtime.target
        )
    };

    BackendCompatibilityCheck {
        code: "runtime_model_pair".to_string(),
        passed,
        message,
    }
}

fn execution_provider(manifest: &PackageManifest) -> String {
    match (manifest.runtime.backend, manifest.runtime.target) {
        (RuntimeBackend::OnnxWebgpu, RuntimeTargetPlatform::BrowserWasmWebgpu) => "webgpu",
        (RuntimeBackend::Candle, RuntimeTargetPlatform::LocalCpu) => "candle-cpu",
        _ => "unsupported",
    }
    .to_string()
}

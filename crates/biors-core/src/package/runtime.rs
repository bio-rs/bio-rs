use super::{
    validate_package_manifest, BackendCapabilitiesSummary, BackendCompatibilityCheck,
    ModelArtifactMetadataSummary, ModelFormat, PackageManifest, RuntimeBackend,
    RuntimeBridgeReport, RuntimeTargetPlatform,
};

/// Build a runtime bridge readiness report from a package manifest.
pub fn plan_runtime_bridge(manifest: &PackageManifest) -> RuntimeBridgeReport {
    let mut blocking_issues = validate_package_manifest(manifest).issues;
    let compatibility_checks = vec![
        runtime_model_pair_check(manifest),
        backend_capabilities_check(manifest),
    ];

    blocking_issues.extend(
        compatibility_checks
            .iter()
            .filter(|check| !check.passed)
            .map(|check| check.message.clone()),
    );

    let backend_id = format!("{}:{}", manifest.name, manifest.runtime.backend);
    let provider = execution_provider(manifest);
    let version = Some(
        manifest
            .runtime
            .version
            .clone()
            .unwrap_or_else(|| format!("{}.v0", manifest.runtime.backend)),
    );
    let backend_config = crate::runtime::BackendConfig {
        backend_id,
        provider: provider.clone(),
        version,
        model_artifact: Some(manifest.model.path.clone()),
    };

    let contract_ready = blocking_issues.is_empty();
    RuntimeBridgeReport {
        ready: contract_ready,
        contract_ready,
        artifact_checked: false,
        execution_ready: false,
        readiness_notes: vec![format!(
            "model artifact '{}' was not format-validated or execution-smoke-tested; ready/contract_ready only indicate manifest and runtime contract compatibility",
            manifest.model.path
        )],
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
        backend_capabilities: Some(backend_capabilities_for(manifest)),
        benchmark_evidence: None,
        regression_baseline: None,
    }
}

fn runtime_model_pair_check(manifest: &PackageManifest) -> BackendCompatibilityCheck {
    if manifest.runtime.backend == RuntimeBackend::ExternalProcess {
        return BackendCompatibilityCheck {
            code: "runtime_model_pair".to_string(),
            passed: false,
            message:
                "external-process is experimental and is not supported by the public package manifest contract"
                    .to_string(),
        };
    }

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

fn backend_capabilities_check(manifest: &PackageManifest) -> BackendCompatibilityCheck {
    let caps = backend_capabilities_for(manifest);
    let passed = true;
    let message = format!(
        "backend capabilities: deterministic={}, supports_batch={}, supports_streaming={}, supported_inputs={:?}, supported_outputs={:?}",
        caps.deterministic,
        caps.supports_batch,
        caps.supports_streaming,
        caps.supported_inputs,
        caps.supported_outputs,
    );
    BackendCompatibilityCheck {
        code: "backend_capabilities".to_string(),
        passed,
        message,
    }
}

fn backend_capabilities_for(manifest: &PackageManifest) -> BackendCapabilitiesSummary {
    match (manifest.runtime.backend, manifest.runtime.target) {
        (RuntimeBackend::Candle, RuntimeTargetPlatform::LocalCpu) => BackendCapabilitiesSummary {
            deterministic: true,
            supports_batch: true,
            supports_streaming: false,
            supported_inputs: vec!["biors.model-input.v0+json".to_string()],
            supported_outputs: vec!["biors.candle.linear-probe.v0+json".to_string()],
        },
        (RuntimeBackend::ExternalProcess, RuntimeTargetPlatform::LocalCpu) => {
            BackendCapabilitiesSummary {
                deterministic: false,
                supports_batch: true,
                supports_streaming: false,
                supported_inputs: vec!["biors.model-input.v0+json".to_string()],
                supported_outputs: vec!["biors.execution-result.v0+json".to_string()],
            }
        }
        (RuntimeBackend::OnnxWebgpu, RuntimeTargetPlatform::BrowserWasmWebgpu) => {
            BackendCapabilitiesSummary {
                deterministic: false,
                supports_batch: true,
                supports_streaming: false,
                supported_inputs: vec!["biors.model-input.v0+json".to_string()],
                supported_outputs: vec!["biors.execution-result.v0+json".to_string()],
            }
        }
        _ => BackendCapabilitiesSummary {
            deterministic: false,
            supports_batch: false,
            supports_streaming: false,
            supported_inputs: vec![],
            supported_outputs: vec![],
        },
    }
}

fn execution_provider(manifest: &PackageManifest) -> String {
    match (manifest.runtime.backend, manifest.runtime.target) {
        (RuntimeBackend::OnnxWebgpu, RuntimeTargetPlatform::BrowserWasmWebgpu) => "webgpu",
        (RuntimeBackend::Candle, RuntimeTargetPlatform::LocalCpu) => "candle-cpu",
        (RuntimeBackend::ExternalProcess, RuntimeTargetPlatform::LocalCpu) => "external-process",
        _ => "unsupported",
    }
    .to_string()
}

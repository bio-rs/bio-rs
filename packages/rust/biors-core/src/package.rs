use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    pub schema_version: String,
    pub name: String,
    pub model: ModelArtifact,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokenizer: Option<TokenAsset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vocab: Option<TokenAsset>,
    pub preprocessing: Vec<PipelineStep>,
    pub postprocessing: Vec<PipelineStep>,
    pub runtime: RuntimeTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_input: Option<DataShape>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_output: Option<DataShape>,
    pub fixtures: Vec<PackageFixture>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelArtifact {
    pub format: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenAsset {
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineStep {
    pub name: String,
    pub implementation: String,
    pub contract: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeTarget {
    pub backend: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageFixture {
    pub name: String,
    pub input: String,
    pub expected_output: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_output_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataShape {
    pub shape: Vec<String>,
    pub dtype: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifestSummary {
    pub schema_version: String,
    pub name: String,
    pub model_format: String,
    pub has_model_checksum: bool,
    pub tokenizer: Option<String>,
    pub vocab: Option<String>,
    pub runtime_backend: String,
    pub runtime_target: String,
    pub preprocessing_steps: usize,
    pub postprocessing_steps: usize,
    pub fixtures: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageValidationReport {
    pub valid: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeBridgeReport {
    pub ready: bool,
    pub backend: String,
    pub target: String,
    pub execution_provider: String,
    pub blocking_issues: Vec<String>,
}

pub fn inspect_package_manifest(manifest: &PackageManifest) -> PackageManifestSummary {
    PackageManifestSummary {
        schema_version: manifest.schema_version.clone(),
        name: manifest.name.clone(),
        model_format: manifest.model.format.clone(),
        has_model_checksum: manifest.model.checksum.is_some(),
        tokenizer: manifest
            .tokenizer
            .as_ref()
            .map(|tokenizer| tokenizer.name.clone()),
        vocab: manifest.vocab.as_ref().map(|vocab| vocab.name.clone()),
        runtime_backend: manifest.runtime.backend.clone(),
        runtime_target: manifest.runtime.target.clone(),
        preprocessing_steps: manifest.preprocessing.len(),
        postprocessing_steps: manifest.postprocessing.len(),
        fixtures: manifest.fixtures.len(),
    }
}

pub fn validate_package_manifest(manifest: &PackageManifest) -> PackageValidationReport {
    let mut issues = Vec::new();

    push_required_issue(&mut issues, "schema_version", &manifest.schema_version);
    if manifest.schema_version != "biors.package.v0" {
        issues.push(format!(
            "schema_version '{}' is not supported; expected 'biors.package.v0'",
            manifest.schema_version
        ));
    }
    push_required_issue(&mut issues, "name", &manifest.name);
    push_required_issue(&mut issues, "model.format", &manifest.model.format);
    push_required_issue(&mut issues, "model.path", &manifest.model.path);
    push_required_issue(&mut issues, "runtime.backend", &manifest.runtime.backend);
    push_required_issue(&mut issues, "runtime.target", &manifest.runtime.target);

    if manifest.fixtures.is_empty() {
        issues.push("fixtures must include at least one fixture".to_string());
    }

    for (index, fixture) in manifest.fixtures.iter().enumerate() {
        push_required_issue(
            &mut issues,
            &format!("fixtures[{index}].name"),
            &fixture.name,
        );
        push_required_issue(
            &mut issues,
            &format!("fixtures[{index}].input"),
            &fixture.input,
        );
        push_required_issue(
            &mut issues,
            &format!("fixtures[{index}].expected_output"),
            &fixture.expected_output,
        );
    }

    if let Some(input) = &manifest.expected_input {
        validate_shape(&mut issues, "expected_input", input);
    }

    if let Some(output) = &manifest.expected_output {
        validate_shape(&mut issues, "expected_output", output);
    }

    PackageValidationReport {
        valid: issues.is_empty(),
        issues,
    }
}

pub fn plan_runtime_bridge(manifest: &PackageManifest) -> RuntimeBridgeReport {
    let mut blocking_issues = validate_package_manifest(manifest).issues;

    let execution_provider = match manifest.runtime.backend.as_str() {
        "onnx-webgpu" => "webgpu",
        unsupported => {
            blocking_issues.push(format!(
                "runtime.backend '{unsupported}' is not supported by the portable bridge"
            ));
            "unsupported"
        }
    };

    if manifest.runtime.target != "browser-wasm-webgpu" {
        blocking_issues.push(format!(
            "runtime.target '{}' is not supported by the portable bridge",
            manifest.runtime.target
        ));
    }

    RuntimeBridgeReport {
        ready: blocking_issues.is_empty(),
        backend: manifest.runtime.backend.clone(),
        target: manifest.runtime.target.clone(),
        execution_provider: execution_provider.to_string(),
        blocking_issues,
    }
}

fn push_required_issue(issues: &mut Vec<String>, field: &str, value: &str) {
    if value.trim().is_empty() {
        issues.push(format!("{field} is required"));
    }
}

fn validate_shape(issues: &mut Vec<String>, field: &str, shape: &DataShape) {
    if shape.shape.is_empty() {
        issues.push(format!("{field}.shape must include at least one dimension"));
    }
    push_required_issue(issues, &format!("{field}.dtype"), &shape.dtype);
}

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

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

pub fn validate_package_manifest_artifacts(
    manifest: &PackageManifest,
    base_dir: &Path,
) -> PackageValidationReport {
    let mut report = validate_package_manifest(manifest);
    validate_artifact(
        &mut report.issues,
        "model",
        &manifest.model.path,
        manifest.model.checksum.as_deref(),
        base_dir,
    );

    if let Some(tokenizer) = &manifest.tokenizer {
        validate_artifact(
            &mut report.issues,
            "tokenizer",
            &tokenizer.path,
            tokenizer.checksum.as_deref(),
            base_dir,
        );
    }

    if let Some(vocab) = &manifest.vocab {
        validate_artifact(
            &mut report.issues,
            "vocab",
            &vocab.path,
            vocab.checksum.as_deref(),
            base_dir,
        );
    }

    for (index, fixture) in manifest.fixtures.iter().enumerate() {
        validate_artifact(
            &mut report.issues,
            &format!("fixtures[{index}].input"),
            &fixture.input,
            fixture.input_hash.as_deref(),
            base_dir,
        );
        validate_artifact(
            &mut report.issues,
            &format!("fixtures[{index}].expected_output"),
            &fixture.expected_output,
            fixture.expected_output_hash.as_deref(),
            base_dir,
        );
    }

    report.valid = report.issues.is_empty();
    report
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

pub fn resolve_package_path(base_dir: &Path, relative_path: &str) -> PathBuf {
    base_dir.join(relative_path)
}

pub fn read_package_file(base_dir: &Path, relative_path: &str) -> Result<Vec<u8>, String> {
    let resolved = resolve_package_path(base_dir, relative_path);
    fs::read(&resolved).map_err(|error| {
        format!(
            "failed to read asset '{}' at '{}': {error}",
            relative_path,
            resolved.display()
        )
    })
}

pub fn sha256_digest(bytes: &[u8]) -> String {
    let normalized = canonical_hash_bytes(bytes);
    let digest = Sha256::digest(&normalized);
    format!("sha256:{digest:x}")
}

pub fn is_sha256_checksum(checksum: &str) -> bool {
    let Some(hex) = checksum.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_artifact(
    issues: &mut Vec<String>,
    field: &str,
    path: &str,
    checksum: Option<&str>,
    base_dir: &Path,
) {
    if path.trim().is_empty() {
        return;
    }

    if let Some(checksum) = checksum {
        if !is_sha256_checksum(checksum) {
            issues.push(format!("{field}.checksum must use sha256:<64 hex>"));
        }
    }

    match read_package_file(base_dir, path) {
        Ok(bytes) => {
            if let Some(checksum) = checksum {
                if is_sha256_checksum(checksum) {
                    let actual = sha256_digest(&bytes);
                    if actual != checksum {
                        issues.push(format!(
                            "{field}.checksum mismatch: expected '{checksum}' but computed '{actual}'"
                        ));
                    }
                }
            }
        }
        Err(error) => issues.push(format!("{field}: {error}")),
    }
}

fn canonical_hash_bytes(bytes: &[u8]) -> Vec<u8> {
    match serde_json::from_slice::<serde_json::Value>(bytes) {
        Ok(json) => serde_json::to_vec(&json).unwrap_or_else(|_| bytes.to_vec()),
        Err(_) => bytes.to_vec(),
    }
}

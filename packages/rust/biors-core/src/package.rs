use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    pub schema_version: SchemaVersion,
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
    pub format: ModelFormat,
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
    pub backend: RuntimeBackend,
    pub target: RuntimeTargetPlatform,
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
    pub dtype: DataType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifestSummary {
    pub schema_version: SchemaVersion,
    pub name: String,
    pub model_format: ModelFormat,
    pub has_model_checksum: bool,
    pub tokenizer: Option<String>,
    pub vocab: Option<String>,
    pub runtime_backend: RuntimeBackend,
    pub runtime_target: RuntimeTargetPlatform,
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
    pub backend: RuntimeBackend,
    pub target: RuntimeTargetPlatform,
    pub execution_provider: String,
    pub blocking_issues: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaVersion {
    #[serde(rename = "biors.package.v0")]
    BiorsPackageV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFormat {
    #[serde(rename = "onnx")]
    Onnx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeBackend {
    #[serde(rename = "onnx-webgpu")]
    OnnxWebgpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeTargetPlatform {
    #[serde(rename = "browser-wasm-webgpu")]
    BrowserWasmWebgpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    #[serde(rename = "uint8")]
    Uint8,
    #[serde(rename = "float32")]
    Float32,
}

pub fn inspect_package_manifest(manifest: &PackageManifest) -> PackageManifestSummary {
    PackageManifestSummary {
        schema_version: manifest.schema_version,
        name: manifest.name.clone(),
        model_format: manifest.model.format,
        has_model_checksum: manifest.model.checksum.is_some(),
        tokenizer: manifest
            .tokenizer
            .as_ref()
            .map(|tokenizer| tokenizer.name.clone()),
        vocab: manifest.vocab.as_ref().map(|vocab| vocab.name.clone()),
        runtime_backend: manifest.runtime.backend,
        runtime_target: manifest.runtime.target,
        preprocessing_steps: manifest.preprocessing.len(),
        postprocessing_steps: manifest.postprocessing.len(),
        fixtures: manifest.fixtures.len(),
    }
}

pub fn validate_package_manifest(manifest: &PackageManifest) -> PackageValidationReport {
    let mut issues = Vec::new();

    push_required_issue(&mut issues, "name", &manifest.name);
    push_required_issue(&mut issues, "model.path", &manifest.model.path);

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
    let blocking_issues = validate_package_manifest(manifest).issues;

    RuntimeBridgeReport {
        ready: blocking_issues.is_empty(),
        backend: manifest.runtime.backend,
        target: manifest.runtime.target,
        execution_provider: "webgpu".to_string(),
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
}

pub fn resolve_package_path(base_dir: &Path, relative_path: &str) -> PathBuf {
    base_dir.join(relative_path)
}

pub fn resolve_package_asset_path(base_dir: &Path, relative_path: &str) -> Result<PathBuf, String> {
    validate_package_relative_path(relative_path)?;
    Ok(resolve_package_path(base_dir, relative_path))
}

pub fn read_package_file(base_dir: &Path, relative_path: &str) -> Result<Vec<u8>, String> {
    let resolved = resolve_package_asset_path(base_dir, relative_path)?;
    fs::read(&resolved).map_err(|error| {
        format!(
            "failed to read asset '{}' at '{}': {error}",
            relative_path,
            resolved.display()
        )
    })
}

pub fn validate_package_relative_path(relative_path: &str) -> Result<(), String> {
    let path = Path::new(relative_path);
    if relative_path.trim().is_empty() {
        return Err("asset path is required".to_string());
    }

    if path.is_absolute() {
        return Err(format!(
            "asset path '{relative_path}' must be relative to the package root"
        ));
    }

    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(format!(
            "asset path '{relative_path}' must stay inside the package root"
        ));
    }

    Ok(())
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

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::BiorsPackageV0 => "biors.package.v0",
        };
        f.write_str(value)
    }
}

impl fmt::Display for ModelFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Onnx => "onnx",
        };
        f.write_str(value)
    }
}

impl fmt::Display for RuntimeBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::OnnxWebgpu => "onnx-webgpu",
        };
        f.write_str(value)
    }
}

impl fmt::Display for RuntimeTargetPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::BrowserWasmWebgpu => "browser-wasm-webgpu",
        };
        f.write_str(value)
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Uint8 => "uint8",
            Self::Float32 => "float32",
        };
        f.write_str(value)
    }
}

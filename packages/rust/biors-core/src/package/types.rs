use serde::{Deserialize, Serialize};
use std::fmt;

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
    pub structured_issues: Vec<PackageValidationIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageValidationIssue {
    pub code: PackageValidationIssueCode,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageValidationIssueCode {
    RequiredField,
    MissingFixture,
    InvalidShape,
    InvalidChecksumFormat,
    ChecksumMismatch,
    InvalidAssetPath,
    AssetReadFailed,
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

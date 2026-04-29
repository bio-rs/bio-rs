use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Portable package manifest for a biological AI model artifact.
pub struct PackageManifest {
    /// Manifest schema version.
    pub schema_version: SchemaVersion,
    /// Human-readable package name.
    pub name: String,
    /// Primary model artifact metadata.
    pub model: ModelArtifact,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional tokenizer artifact metadata.
    pub tokenizer: Option<TokenAsset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional vocabulary artifact metadata.
    pub vocab: Option<TokenAsset>,
    /// Ordered preprocessing contract steps.
    pub preprocessing: Vec<PipelineStep>,
    /// Ordered postprocessing contract steps.
    pub postprocessing: Vec<PipelineStep>,
    /// Runtime backend and target expected by the package.
    pub runtime: RuntimeTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional expected input shape and dtype.
    pub expected_input: Option<DataShape>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional expected output shape and dtype.
    pub expected_output: Option<DataShape>,
    /// Verification fixtures bundled with the package.
    pub fixtures: Vec<PackageFixture>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Model artifact referenced by a package manifest.
pub struct ModelArtifact {
    /// Model file format.
    pub format: ModelFormat,
    /// Package-relative artifact path.
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional `sha256:<64 hex>` checksum for the artifact.
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Tokenizer or vocabulary artifact referenced by a package manifest.
pub struct TokenAsset {
    /// Human-readable asset name.
    pub name: String,
    /// Package-relative artifact path.
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional `sha256:<64 hex>` checksum for the asset.
    pub checksum: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional contract version implemented by the asset.
    pub contract_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Named preprocessing or postprocessing contract step.
pub struct PipelineStep {
    /// Step name.
    pub name: String,
    /// Implementation provider or tool name.
    pub implementation: String,
    /// Contract implemented by the step.
    pub contract: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional contract version.
    pub contract_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Runtime backend and target platform requested by a package.
pub struct RuntimeTarget {
    /// Runtime backend identifier.
    pub backend: RuntimeBackend,
    /// Target platform identifier.
    pub target: RuntimeTargetPlatform,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Input and expected-output pair used for package verification.
pub struct PackageFixture {
    /// Fixture name used to match observations.
    pub name: String,
    /// Package-relative fixture input path.
    pub input: String,
    /// Package-relative expected output path.
    pub expected_output: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional checksum for fixture input.
    pub input_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional checksum for expected output.
    pub expected_output_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Tensor shape and dtype contract.
pub struct DataShape {
    /// Shape dimensions represented as strings to allow symbolic dimensions.
    pub shape: Vec<String>,
    /// Element dtype.
    pub dtype: DataType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Compact manifest summary for inspect-style outputs.
pub struct PackageManifestSummary {
    /// Manifest schema version.
    pub schema_version: SchemaVersion,
    /// Package name.
    pub name: String,
    /// Model file format.
    pub model_format: ModelFormat,
    /// Whether the model artifact declares a checksum.
    pub has_model_checksum: bool,
    /// Optional tokenizer asset name.
    pub tokenizer: Option<String>,
    /// Optional vocabulary asset name.
    pub vocab: Option<String>,
    /// Runtime backend.
    pub runtime_backend: RuntimeBackend,
    /// Runtime target platform.
    pub runtime_target: RuntimeTargetPlatform,
    /// Number of preprocessing steps.
    pub preprocessing_steps: usize,
    /// Number of postprocessing steps.
    pub postprocessing_steps: usize,
    /// Number of package verification fixtures.
    pub fixtures: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Manifest validation result with human and structured issue forms.
pub struct PackageValidationReport {
    /// True when no structured validation issues were produced.
    pub valid: bool,
    /// Human-readable issue messages retained for compatibility.
    pub issues: Vec<String>,
    /// Machine-readable issue details.
    pub structured_issues: Vec<PackageValidationIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// One structured manifest validation issue.
pub struct PackageValidationIssue {
    /// Stable issue code.
    pub code: PackageValidationIssueCode,
    /// Manifest field path associated with the issue.
    pub field: String,
    /// Human-readable explanation.
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Stable manifest validation issue codes.
pub enum PackageValidationIssueCode {
    /// Required string field is empty.
    RequiredField,
    /// Manifest has no fixtures.
    MissingFixture,
    /// Shape contract has no dimensions.
    InvalidShape,
    /// Checksum is not `sha256:<64 hex>`.
    InvalidChecksumFormat,
    /// Artifact checksum does not match computed content hash.
    ChecksumMismatch,
    /// Package-relative asset path is empty, absolute, or escapes the package root.
    InvalidAssetPath,
    /// Asset could not be read from disk.
    AssetReadFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Runtime bridge readiness report for a package manifest.
pub struct RuntimeBridgeReport {
    /// True when validation produced no blocking issues.
    pub ready: bool,
    /// Runtime backend.
    pub backend: RuntimeBackend,
    /// Target runtime platform.
    pub target: RuntimeTargetPlatform,
    /// Execution provider selected for the bridge.
    pub execution_provider: String,
    /// Human-readable blocking validation issues.
    pub blocking_issues: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Supported package manifest schema versions.
pub enum SchemaVersion {
    #[serde(rename = "biors.package.v0")]
    BiorsPackageV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Supported model artifact formats.
pub enum ModelFormat {
    #[serde(rename = "onnx")]
    Onnx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Supported runtime backends.
pub enum RuntimeBackend {
    #[serde(rename = "onnx-webgpu")]
    OnnxWebgpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Supported runtime target platforms.
pub enum RuntimeTargetPlatform {
    #[serde(rename = "browser-wasm-webgpu")]
    BrowserWasmWebgpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Supported tensor element dtypes.
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

impl PackageValidationReport {
    pub(crate) fn push_issue(
        &mut self,
        code: PackageValidationIssueCode,
        field: &str,
        message: &str,
    ) {
        self.issues.push(message.to_string());
        self.structured_issues.push(PackageValidationIssue {
            code,
            field: field.to_string(),
            message: message.to_string(),
        });
    }

    pub(crate) fn finish(mut self) -> Self {
        self.valid = self.structured_issues.is_empty();
        self
    }
}

impl Default for PackageValidationReport {
    fn default() -> Self {
        Self {
            valid: true,
            issues: Vec::new(),
            structured_issues: Vec::new(),
        }
    }
}

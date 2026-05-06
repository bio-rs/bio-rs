use super::{DataType, ModelFormat, RuntimeBackend, RuntimeTargetPlatform, SchemaVersion};
use serde::{Deserialize, Serialize};

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

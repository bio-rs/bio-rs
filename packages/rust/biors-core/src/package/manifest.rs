use super::{DataType, ModelFormat, RuntimeBackend, RuntimeTargetPlatform, SchemaVersion};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Portable package manifest for a biological AI model artifact.
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
    /// Optional `sha256:<64 hex>` checksum for the artifact.
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenAsset {
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional `sha256:<64 hex>` checksum for the asset.
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
    /// Shape dimensions represented as strings to allow symbolic dimensions.
    pub shape: Vec<String>,
    pub dtype: DataType,
}

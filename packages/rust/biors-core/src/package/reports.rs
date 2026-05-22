use super::{
    ModelArtifactMetadata, ModelFormat, RuntimeBackend, RuntimeTargetPlatform, SchemaVersion,
};
use crate::runtime::BackendConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Compact manifest summary for inspect-style outputs.
pub struct PackageManifestSummary {
    pub schema_version: SchemaVersion,
    pub name: String,
    pub package_layout: Option<PackageDirectoryLayoutSummary>,
    pub metadata: Option<PackageMetadataSummary>,
    pub model_format: ModelFormat,
    pub has_model_checksum: bool,
    pub model_metadata: Option<ModelArtifactMetadataSummary>,
    pub tokenizer: Option<String>,
    pub vocab: Option<String>,
    pub runtime_backend: RuntimeBackend,
    pub runtime_target: RuntimeTargetPlatform,
    pub preprocessing_steps: usize,
    pub postprocessing_steps: usize,
    pub fixtures: usize,
    /// Draft package layout paths grouped for inspect UX and future package layers.
    pub layout: PackageLayoutSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Declared package directory layout returned by inspect.
pub struct PackageDirectoryLayoutSummary {
    pub manifest: String,
    pub models: String,
    pub tokenizers: Option<String>,
    pub vocabs: Option<String>,
    pub pipelines: Option<String>,
    pub fixtures: String,
    pub observed: Option<String>,
    pub docs: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Compact research metadata returned by inspect.
pub struct PackageMetadataSummary {
    pub license: String,
    pub citation: String,
    pub model_card: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Compact model artifact metadata returned by inspect and runtime planning.
pub struct ModelArtifactMetadataSummary {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Package-relative layout paths declared by a manifest.
pub struct PackageLayoutSummary {
    pub model: String,
    pub tokenizer: Option<String>,
    pub vocab: Option<String>,
    pub pipeline_configs: Vec<String>,
    pub fixture_inputs: Vec<String>,
    pub fixture_outputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Manifest validation result with human and structured issue forms.
pub struct PackageValidationReport {
    pub valid: bool,
    /// Human-readable issue messages retained for compatibility.
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
    /// Asset path does not live under the declared v1 package layout directory.
    LayoutMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Runtime bridge readiness report for a package manifest.
pub struct RuntimeBridgeReport {
    pub ready: bool,
    pub backend: RuntimeBackend,
    pub target: RuntimeTargetPlatform,
    pub model_format: ModelFormat,
    pub model_metadata: Option<ModelArtifactMetadataSummary>,
    pub backend_config: BackendConfig,
    pub execution_provider: String,
    pub compatibility_checks: Vec<BackendCompatibilityCheck>,
    pub blocking_issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// One deterministic runtime compatibility decision.
pub struct BackendCompatibilityCheck {
    pub code: String,
    pub passed: bool,
    pub message: String,
}

impl From<&ModelArtifactMetadata> for ModelArtifactMetadataSummary {
    fn from(metadata: &ModelArtifactMetadata) -> Self {
        Self {
            name: metadata.name.clone(),
            version: metadata.version.clone(),
            architecture: metadata.architecture.clone(),
            task: metadata.task.clone(),
            source: metadata.source.clone(),
            description: metadata.description.clone(),
        }
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

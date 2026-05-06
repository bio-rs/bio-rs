use super::{ModelFormat, RuntimeBackend, RuntimeTargetPlatform, SchemaVersion};
use serde::{Deserialize, Serialize};

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

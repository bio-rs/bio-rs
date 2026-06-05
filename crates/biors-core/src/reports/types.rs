use serde::{Deserialize, Serialize};

/// Stable schema identifier for shareable report JSON exports.
pub const REPORT_SCHEMA_VERSION: &str = "biors.report.v0";

/// JSON container shape used as report input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportInputContainer {
    RawJson,
    CliSuccessEnvelope,
    CliErrorEnvelope,
}

impl ReportInputContainer {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RawJson => "raw_json",
            Self::CliSuccessEnvelope => "cli_success_envelope",
            Self::CliErrorEnvelope => "cli_error_envelope",
        }
    }
}

/// Detected bio-rs payload family used to render report sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportInputKind {
    CliError,
    BioEntityExport,
    SequenceWorkflowOutput,
    ValidationReport,
    GenericJson,
}

impl ReportInputKind {
    pub const fn title(self) -> &'static str {
        match self {
            Self::CliError => "bio-rs CLI Error Report",
            Self::BioEntityExport => "bio-rs Conversion Report",
            Self::SequenceWorkflowOutput => "bio-rs Workflow Report",
            Self::ValidationReport => "bio-rs Validation Report",
            Self::GenericJson => "bio-rs JSON Report",
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CliError => "cli_error",
            Self::BioEntityExport => "bio_entity_export",
            Self::SequenceWorkflowOutput => "sequence_workflow_output",
            Self::ValidationReport => "validation_report",
            Self::GenericJson => "generic_json",
        }
    }
}

/// Overall status derived from the source JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    Pass,
    Warning,
    Fail,
    Unknown,
}

impl ReportStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warning => "warning",
            Self::Fail => "fail",
            Self::Unknown => "unknown",
        }
    }
}

/// One stable label/value pair rendered in JSON and Markdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportMetric {
    pub label: String,
    pub value: String,
}

impl ReportMetric {
    pub fn new(label: impl Into<String>, value: impl ToString) -> Self {
        Self {
            label: label.into(),
            value: value.to_string(),
        }
    }
}

/// A deterministic human-readable report section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportSection {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub metrics: Vec<ReportMetric>,
    pub items: Vec<String>,
}

impl ReportSection {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        metrics: Vec<ReportMetric>,
        items: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            metrics,
            items,
        }
    }
}

/// Deterministic report provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportProvenance {
    pub biors_core_version: String,
    pub generator: String,
    pub input_container: ReportInputContainer,
    pub input_kind: ReportInputKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_schema_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_biors_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_input_hash: Option<String>,
    pub input_raw_sha256: String,
    pub input_canonical_sha256: String,
    pub report_markdown_sha256: String,
}

/// Shareable report export containing machine-readable summary and Markdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareableReport {
    pub schema_version: String,
    pub title: String,
    pub summary: String,
    pub status: ReportStatus,
    pub provenance: ReportProvenance,
    pub sections: Vec<ReportSection>,
    pub human_report: String,
}

/// Error returned when report input cannot be parsed.
#[derive(Debug)]
pub enum ReportBuildError {
    InvalidJson(serde_json::Error),
}

impl ReportBuildError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidJson(_) => "report.invalid_json",
        }
    }
}

impl std::fmt::Display for ReportBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(error) => write!(f, "report input is not valid JSON: {error}"),
        }
    }
}

impl std::error::Error for ReportBuildError {}

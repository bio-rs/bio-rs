use serde_json::Value;

use super::markdown::render_markdown;
use super::sections::sections_for;
use super::types::{
    ReportBuildError, ReportInputContainer, ReportInputKind, ReportProvenance, ReportStatus,
    ShareableReport, REPORT_SCHEMA_VERSION,
};
use crate::hash::{sha256_bytes_digest, sha256_canonical_json_digest};

struct SourcePayload<'a> {
    container: ReportInputContainer,
    payload: &'a Value,
    source_biors_version: Option<String>,
    source_input_hash: Option<String>,
}

/// Build a deterministic shareable report from a bio-rs JSON payload.
pub fn build_shareable_report_from_json_bytes(
    input: &[u8],
) -> Result<ShareableReport, ReportBuildError> {
    let value = serde_json::from_slice::<Value>(input).map_err(ReportBuildError::InvalidJson)?;
    let source = source_payload(&value);
    let kind = detect_kind(&source);
    let status = status_for(kind, source.payload);
    let sections = sections_for(kind, source.payload);
    let title = kind.title().to_string();
    let summary = summary_for(kind, status, source.payload);
    let mut provenance = ReportProvenance {
        biors_core_version: env!("CARGO_PKG_VERSION").to_string(),
        generator: REPORT_SCHEMA_VERSION.to_string(),
        input_container: source.container,
        input_kind: kind,
        source_schema_version: string_field(source.payload, "schema_version"),
        source_biors_version: source.source_biors_version,
        source_input_hash: source.source_input_hash,
        input_raw_sha256: sha256_bytes_digest(input),
        input_canonical_sha256: sha256_canonical_json_digest(input),
        report_markdown_sha256: String::new(),
    };
    let human_report = render_markdown(&title, &summary, status, &provenance, &sections);
    provenance.report_markdown_sha256 = sha256_bytes_digest(human_report.as_bytes());

    Ok(ShareableReport {
        schema_version: REPORT_SCHEMA_VERSION.to_string(),
        title,
        summary,
        status,
        provenance,
        sections,
        human_report,
    })
}

fn source_payload(value: &Value) -> SourcePayload<'_> {
    let Some(object) = value.as_object() else {
        return raw_source(value);
    };
    match object.get("ok").and_then(Value::as_bool) {
        Some(true) => object.get("data").map_or_else(
            || raw_source(value),
            |payload| SourcePayload {
                container: ReportInputContainer::CliSuccessEnvelope,
                payload,
                source_biors_version: string_field(value, "biors_version"),
                source_input_hash: string_field(value, "input_hash"),
            },
        ),
        Some(false) => object.get("error").map_or_else(
            || raw_source(value),
            |payload| SourcePayload {
                container: ReportInputContainer::CliErrorEnvelope,
                payload,
                source_biors_version: None,
                source_input_hash: None,
            },
        ),
        None => raw_source(value),
    }
}

fn raw_source(value: &Value) -> SourcePayload<'_> {
    SourcePayload {
        container: ReportInputContainer::RawJson,
        payload: value,
        source_biors_version: None,
        source_input_hash: None,
    }
}

fn detect_kind(source: &SourcePayload<'_>) -> ReportInputKind {
    if source.container == ReportInputContainer::CliErrorEnvelope {
        return ReportInputKind::CliError;
    }
    if string_field(source.payload, "schema_version").as_deref() == Some("biors.conversion.v0") {
        return ReportInputKind::BioEntityExport;
    }
    if source.payload.get("workflow").is_some()
        && source.payload.get("provenance").is_some()
        && source.payload.get("tokenization").is_some()
    {
        return ReportInputKind::SequenceWorkflowOutput;
    }
    if source
        .payload
        .get("valid")
        .and_then(Value::as_bool)
        .is_some()
        && source
            .payload
            .get("records")
            .and_then(Value::as_u64)
            .is_some()
    {
        return ReportInputKind::ValidationReport;
    }
    ReportInputKind::GenericJson
}

fn status_for(kind: ReportInputKind, value: &Value) -> ReportStatus {
    match kind {
        ReportInputKind::CliError => ReportStatus::Fail,
        ReportInputKind::BioEntityExport => {
            let records = usize_field(value, "records");
            let ready = usize_field(value, "model_ready_records");
            if usize_field(value, "error_count") > 0 || ready < records {
                ReportStatus::Fail
            } else if usize_field(value, "warning_count") > 0 {
                ReportStatus::Warning
            } else {
                ReportStatus::Pass
            }
        }
        ReportInputKind::SequenceWorkflowOutput => {
            if !bool_field(value, "model_ready").unwrap_or(false) {
                ReportStatus::Fail
            } else {
                let validation = value.get("validation").unwrap_or(value);
                warning_status(validation)
            }
        }
        ReportInputKind::ValidationReport => {
            if !bool_field(value, "valid").unwrap_or(false) || usize_field(value, "error_count") > 0
            {
                ReportStatus::Fail
            } else {
                warning_status(value)
            }
        }
        ReportInputKind::GenericJson => ReportStatus::Unknown,
    }
}

fn warning_status(value: &Value) -> ReportStatus {
    if usize_field(value, "warning_count") > 0 {
        ReportStatus::Warning
    } else {
        ReportStatus::Pass
    }
}

fn summary_for(kind: ReportInputKind, status: ReportStatus, value: &Value) -> String {
    match kind {
        ReportInputKind::CliError => format!(
            "Command failed with code '{}'.",
            string_field(value, "code").unwrap_or_else(|| "unknown".to_string())
        ),
        ReportInputKind::BioEntityExport => format!(
            "Converted {} records; {} valid; {} model-ready; {} warnings; {} errors.",
            usize_field(value, "records"),
            usize_field(value, "valid_records"),
            usize_field(value, "model_ready_records"),
            usize_field(value, "warning_count"),
            usize_field(value, "error_count")
        ),
        ReportInputKind::SequenceWorkflowOutput => format!(
            "Workflow '{}' finished with status '{}'.",
            string_field(value, "workflow").unwrap_or_else(|| "unknown".to_string()),
            status.as_str()
        ),
        ReportInputKind::ValidationReport => format!(
            "Validated {} records; {} valid; {} warnings; {} errors.",
            usize_field(value, "records"),
            usize_field(value, "valid_records"),
            usize_field(value, "warning_count"),
            usize_field(value, "error_count")
        ),
        ReportInputKind::GenericJson => format!(
            "Parsed JSON input with {} top-level fields.",
            value.as_object().map_or(0, serde_json::Map::len)
        ),
    }
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    value.get(field).and_then(Value::as_str).map(str::to_string)
}

fn bool_field(value: &Value, field: &str) -> Option<bool> {
    value.get(field).and_then(Value::as_bool)
}

fn usize_field(value: &Value, field: &str) -> usize {
    value.get(field).and_then(Value::as_u64).unwrap_or(0) as usize
}

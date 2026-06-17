use biors_core::{
    sequence::KindAwareSequenceValidationReport,
    tokenizer::{summarize_tokenized_proteins, TokenizedProtein},
    workflow::SequenceWorkflowOutput,
};
use serde_json::{json, Value};

const COMPACT_FASTA_TEXT_BYTES: usize = 2048;

pub(crate) fn should_compact_fasta(fasta_text: &str, include_full_payload: bool) -> bool {
    !include_full_payload && fasta_text.len() > COMPACT_FASTA_TEXT_BYTES
}

pub(crate) fn compact_tokenize_response(records: &[TokenizedProtein]) -> Value {
    json!({
        "schema_version": "biors.mcp.compact.v0",
        "tool": "tokenize",
        "compact": true,
        "full_payload_requested_with": "include_records",
        "summary": summarize_tokenized_proteins(records),
        "issues": tokenization_issues(records),
    })
}

pub(crate) fn compact_validate_response(report: &KindAwareSequenceValidationReport) -> Value {
    let issues: Vec<_> = report
        .sequences
        .iter()
        .filter(|record| !record.warnings.is_empty() || !record.errors.is_empty())
        .map(|record| {
            json!({
                "id": record.id,
                "kind": record.kind,
                "warning_count": record.warnings.len(),
                "error_count": record.errors.len(),
            })
        })
        .collect();

    json!({
        "schema_version": "biors.mcp.compact.v0",
        "tool": "validate",
        "compact": true,
        "full_payload_requested_with": "include_records",
        "summary": {
            "records": report.records,
            "valid_records": report.valid_records,
            "warning_count": report.warning_count,
            "error_count": report.error_count,
            "kind_counts": report.kind_counts,
        },
        "issues": issues,
    })
}

pub(crate) fn compact_workflow_response(output: &SequenceWorkflowOutput) -> Value {
    json!({
        "schema_version": "biors.mcp.compact.v0",
        "tool": "workflow",
        "compact": true,
        "full_payload_requested_with": "include_payload",
        "workflow": output.workflow,
        "model_ready": output.model_ready,
        "input_hash": output.provenance.input_hash,
        "validation_summary": {
            "records": output.validation.records,
            "valid_records": output.validation.valid_records,
            "warning_count": output.validation.warning_count,
            "error_count": output.validation.error_count,
        },
        "tokenization_summary": output.tokenization.summary,
        "readiness_issues": output.readiness_issues,
    })
}

fn tokenization_issues(records: &[TokenizedProtein]) -> Vec<Value> {
    records
        .iter()
        .filter(|record| !record.warnings.is_empty() || !record.errors.is_empty())
        .map(|record| {
            json!({
                "id": record.id,
                "warning_count": record.warnings.len(),
                "error_count": record.errors.len(),
            })
        })
        .collect()
}

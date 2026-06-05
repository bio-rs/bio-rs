use std::collections::BTreeMap;

use serde_json::Value;

use super::types::{ReportInputKind, ReportMetric, ReportSection};

pub(super) fn sections_for(kind: ReportInputKind, value: &Value) -> Vec<ReportSection> {
    match kind {
        ReportInputKind::CliError => cli_error_sections(value),
        ReportInputKind::BioEntityExport => conversion_sections(value),
        ReportInputKind::SequenceWorkflowOutput => workflow_sections(value),
        ReportInputKind::ValidationReport => validation_sections(value),
        ReportInputKind::GenericJson => generic_sections(value),
    }
}

fn cli_error_sections(value: &Value) -> Vec<ReportSection> {
    vec![ReportSection::new(
        "cli_error",
        "CLI Error",
        string_field(value, "message").unwrap_or_else(|| "No error message was provided.".into()),
        vec![
            ReportMetric::new("code", string_field(value, "code").unwrap_or_default()),
            ReportMetric::new("has_details", value.get("details").is_some()),
        ],
        Vec::new(),
    )]
}

fn conversion_sections(value: &Value) -> Vec<ReportSection> {
    let mut entity_counts = BTreeMap::<String, usize>::new();
    let mut issues = Vec::new();
    for entity in array_field(value, "entities") {
        if let Some(entity_type) = string_field(entity, "entity_type") {
            *entity_counts.entry(entity_type).or_default() += 1;
        }
        collect_conversion_issues(entity, &mut issues);
    }
    let metrics = entity_counts
        .into_iter()
        .map(|(label, count)| ReportMetric::new(format!("{label} records"), count))
        .collect();
    vec![
        ReportSection::new(
            "record_counts",
            "Record Counts",
            "Aggregate conversion counts from the JSON export.",
            vec![
                ReportMetric::new("records", usize_field(value, "records")),
                ReportMetric::new("valid_records", usize_field(value, "valid_records")),
                ReportMetric::new(
                    "model_ready_records",
                    usize_field(value, "model_ready_records"),
                ),
                ReportMetric::new("warning_count", usize_field(value, "warning_count")),
                ReportMetric::new("error_count", usize_field(value, "error_count")),
            ],
            Vec::new(),
        ),
        ReportSection::new(
            "entity_types",
            "Entity Types",
            "Record family counts preserved in the conversion export.",
            metrics,
            Vec::new(),
        ),
        ReportSection::new(
            "issues",
            "Issues",
            "First conversion warnings and errors, capped for a compact report.",
            Vec::new(),
            cap_items(issues),
        ),
    ]
}

fn workflow_sections(value: &Value) -> Vec<ReportSection> {
    let validation = value.get("validation").unwrap_or(value);
    let tokenization = value.get("tokenization").unwrap_or(&Value::Null);
    let summary = tokenization.get("summary").unwrap_or(&Value::Null);
    vec![
        ReportSection::new(
            "workflow",
            "Workflow",
            "Model-readiness and workflow identity.",
            vec![
                ReportMetric::new(
                    "workflow",
                    string_field(value, "workflow").unwrap_or_default(),
                ),
                ReportMetric::new(
                    "model_ready",
                    bool_field(value, "model_ready").unwrap_or(false),
                ),
                ReportMetric::new(
                    "readiness_issues",
                    array_field(value, "readiness_issues").len(),
                ),
            ],
            readiness_items(value),
        ),
        ReportSection::new(
            "validation",
            "Validation",
            "Sequence validation counts captured before tokenization.",
            validation_metrics(validation),
            Vec::new(),
        ),
        ReportSection::new(
            "tokenization",
            "Tokenization",
            "Tokenizer output summary fields from the workflow payload.",
            object_metrics(summary),
            Vec::new(),
        ),
    ]
}

fn validation_sections(value: &Value) -> Vec<ReportSection> {
    vec![ReportSection::new(
        "validation",
        "Validation",
        "Validation counts from the source report.",
        validation_metrics(value),
        collect_named_issue_items(value),
    )]
}

fn generic_sections(value: &Value) -> Vec<ReportSection> {
    let items = value
        .as_object()
        .map(|object| {
            object
                .keys()
                .map(|key| format!("top-level field: {key}"))
                .collect()
        })
        .unwrap_or_default();
    vec![ReportSection::new(
        "json",
        "JSON Summary",
        "The input is valid JSON but does not match a specialized bio-rs report shape.",
        object_metrics(value),
        cap_items(items),
    )]
}

fn validation_metrics(value: &Value) -> Vec<ReportMetric> {
    [
        "valid",
        "records",
        "valid_records",
        "warning_count",
        "error_count",
    ]
    .into_iter()
    .filter_map(|field| {
        value
            .get(field)
            .map(|item| ReportMetric::new(field, value_text(item)))
    })
    .collect()
}

fn object_metrics(value: &Value) -> Vec<ReportMetric> {
    let Some(object) = value.as_object() else {
        return Vec::new();
    };
    object
        .iter()
        .filter(|(_, value)| value.is_boolean() || value.is_number() || value.is_string())
        .map(|(key, value)| ReportMetric::new(key, value_text(value)))
        .collect()
}

fn readiness_items(value: &Value) -> Vec<String> {
    array_field(value, "readiness_issues")
        .iter()
        .map(|issue| {
            format!(
                "{}: {}",
                string_field(issue, "id").unwrap_or_else(|| "record".into()),
                string_field(issue, "message").unwrap_or_else(|| "not model-ready".into())
            )
        })
        .collect()
}

fn collect_conversion_issues(entity: &Value, issues: &mut Vec<String>) {
    let id = string_field(entity, "id").unwrap_or_else(|| "record".into());
    let Some(validation) = entity.get("validation") else {
        return;
    };
    for field in ["errors", "warnings"] {
        for issue in array_field(validation, field) {
            issues.push(format!(
                "{} {} {}: {}",
                id,
                string_field(issue, "severity").unwrap_or_else(|| field.into()),
                string_field(issue, "code").unwrap_or_else(|| "issue".into()),
                string_field(issue, "message").unwrap_or_default()
            ));
        }
    }
}

fn collect_named_issue_items(value: &Value) -> Vec<String> {
    let mut items = Vec::new();
    for field in ["errors", "warnings", "issues"] {
        for issue in array_field(value, field) {
            items.push(value_text(issue));
        }
    }
    cap_items(items)
}

fn cap_items(mut items: Vec<String>) -> Vec<String> {
    items.truncate(10);
    items
}

fn array_field<'a>(value: &'a Value, field: &str) -> &'a [Value] {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
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

fn value_text(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

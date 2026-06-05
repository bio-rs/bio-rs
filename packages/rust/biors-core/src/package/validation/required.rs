use super::super::{PackageValidationIssueCode, PackageValidationReport};

pub(super) fn push_required_issue(report: &mut PackageValidationReport, field: &str, value: &str) {
    if value.trim().is_empty() {
        report.push_issue(
            PackageValidationIssueCode::RequiredField,
            field,
            &format!("{field} is required"),
        );
    }
}

pub(super) fn push_required_option_issue(
    report: &mut PackageValidationReport,
    field: &str,
    value: Option<&String>,
) {
    match value {
        Some(value) => push_required_issue(report, field, value),
        None => report.push_issue(
            PackageValidationIssueCode::RequiredField,
            field,
            &format!("{field} is required"),
        ),
    }
}

pub(super) fn push_required_list_issue(
    report: &mut PackageValidationReport,
    field: &str,
    value: &[String],
) {
    if value.is_empty() || value.iter().any(|entry| entry.trim().is_empty()) {
        report.push_issue(
            PackageValidationIssueCode::RequiredField,
            field,
            &format!("{field} must include non-empty values"),
        );
    }
}

use super::{SequenceWorkflowReadinessIssue, READINESS_ISSUE_CODE};
use crate::tokenizer::TokenizedProtein;

pub(super) fn readiness_issues(
    tokenized: &[TokenizedProtein],
) -> Vec<SequenceWorkflowReadinessIssue> {
    tokenized
        .iter()
        .filter(|record| {
            record.tokens.is_empty() || !record.warnings.is_empty() || !record.errors.is_empty()
        })
        .map(|record| {
            let warning_count = record.warnings.len();
            let error_count = record.errors.len();
            let message = if record.tokens.is_empty() {
                format!(
                    "sequence '{}' is not model-ready: empty sequences cannot be converted into model input",
                    record.id
                )
            } else {
                format!(
                    "sequence '{}' is not model-ready: {warning_count} warnings and {error_count} errors must be resolved before model-input generation",
                    record.id
                )
            };
            SequenceWorkflowReadinessIssue {
                code: READINESS_ISSUE_CODE.to_string(),
                id: record.id.clone(),
                warning_count,
                error_count,
                message,
            }
        })
        .collect()
}

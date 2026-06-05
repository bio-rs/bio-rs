use super::super::{DataShape, PackageValidationIssueCode, PackageValidationReport};

pub(super) fn validate_shape(report: &mut PackageValidationReport, field: &str, shape: &DataShape) {
    if shape.shape.is_empty() {
        report.push_issue(
            PackageValidationIssueCode::InvalidShape,
            &format!("{field}.shape"),
            &format!("{field}.shape must include at least one dimension"),
        );
    }
    for (index, dimension) in shape.shape.iter().enumerate() {
        if dimension.trim().is_empty() {
            let field = format!("{field}.shape[{index}]");
            report.push_issue(
                PackageValidationIssueCode::InvalidShape,
                &field,
                &format!("{field} must be a non-empty dimension"),
            );
        }
    }
}

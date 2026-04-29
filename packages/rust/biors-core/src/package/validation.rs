use super::{DataShape, PackageManifest, PackageValidationIssueCode, PackageValidationReport};

/// Validate package manifest fields that do not require filesystem access.
pub fn validate_package_manifest(manifest: &PackageManifest) -> PackageValidationReport {
    let mut report = PackageValidationReport::default();

    push_required_issue(&mut report, "name", &manifest.name);
    push_required_issue(&mut report, "model.path", &manifest.model.path);
    validate_fixture_list(&mut report, manifest);
    validate_optional_shape(
        &mut report,
        "expected_input",
        manifest.expected_input.as_ref(),
    );
    validate_optional_shape(
        &mut report,
        "expected_output",
        manifest.expected_output.as_ref(),
    );

    report.finish()
}

fn validate_fixture_list(report: &mut PackageValidationReport, manifest: &PackageManifest) {
    if manifest.fixtures.is_empty() {
        report.push_issue(
            PackageValidationIssueCode::MissingFixture,
            "fixtures",
            "fixtures must include at least one fixture",
        );
        return;
    }

    for (index, fixture) in manifest.fixtures.iter().enumerate() {
        push_required_issue(report, &format!("fixtures[{index}].name"), &fixture.name);
        push_required_issue(report, &format!("fixtures[{index}].input"), &fixture.input);
        push_required_issue(
            report,
            &format!("fixtures[{index}].expected_output"),
            &fixture.expected_output,
        );
    }
}

fn validate_optional_shape(
    report: &mut PackageValidationReport,
    field: &str,
    shape: Option<&DataShape>,
) {
    if let Some(shape) = shape {
        validate_shape(report, field, shape);
    }
}

fn push_required_issue(report: &mut PackageValidationReport, field: &str, value: &str) {
    if value.trim().is_empty() {
        report.push_issue(
            PackageValidationIssueCode::RequiredField,
            field,
            &format!("{field} is required"),
        );
    }
}

fn validate_shape(report: &mut PackageValidationReport, field: &str, shape: &DataShape) {
    if shape.shape.is_empty() {
        report.push_issue(
            PackageValidationIssueCode::InvalidShape,
            &format!("{field}.shape"),
            &format!("{field}.shape must include at least one dimension"),
        );
    }
}

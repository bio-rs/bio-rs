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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::{
        DataShape, DataType, ModelArtifact, ModelFormat, PackageFixture, PackageManifest,
        RuntimeBackend, RuntimeTarget, RuntimeTargetPlatform, SchemaVersion,
    };

    fn minimal_manifest() -> PackageManifest {
        PackageManifest {
            schema_version: SchemaVersion::BiorsPackageV0,
            name: "test-pkg".into(),
            model: ModelArtifact {
                format: ModelFormat::Onnx,
                path: "model.onnx".into(),
                checksum: None,
            },
            tokenizer: None,
            vocab: None,
            preprocessing: vec![],
            postprocessing: vec![],
            runtime: RuntimeTarget {
                backend: RuntimeBackend::OnnxWebgpu,
                target: RuntimeTargetPlatform::BrowserWasmWebgpu,
            },
            expected_input: None,
            expected_output: None,
            fixtures: vec![PackageFixture {
                name: "fixture1".into(),
                input: "input.json".into(),
                expected_output: "output.json".into(),
                input_hash: None,
                expected_output_hash: None,
            }],
        }
    }

    #[test]
    fn validate_package_manifest_accepts_minimal_valid_manifest() {
        let manifest = minimal_manifest();
        let report = validate_package_manifest(&manifest);
        assert!(report.valid);
        assert!(report.issues.is_empty());
        assert!(report.structured_issues.is_empty());
    }

    #[test]
    fn validate_package_manifest_rejects_missing_name() {
        let mut manifest = minimal_manifest();
        manifest.name = "".into();
        let report = validate_package_manifest(&manifest);
        assert!(!report.valid);
        assert_eq!(report.structured_issues.len(), 1);
        assert_eq!(report.structured_issues[0].code, PackageValidationIssueCode::RequiredField);
        assert_eq!(report.structured_issues[0].field, "name");
    }

    #[test]
    fn validate_package_manifest_rejects_missing_model_path() {
        let mut manifest = minimal_manifest();
        manifest.model.path = "   ".into();
        let report = validate_package_manifest(&manifest);
        assert!(!report.valid);
        let model_issue = report
            .structured_issues
            .iter()
            .find(|i| i.field == "model.path")
            .expect("model.path issue expected");
        assert_eq!(model_issue.code, PackageValidationIssueCode::RequiredField);
    }

    #[test]
    fn validate_package_manifest_rejects_empty_fixtures() {
        let mut manifest = minimal_manifest();
        manifest.fixtures.clear();
        let report = validate_package_manifest(&manifest);
        assert!(!report.valid);
        assert_eq!(report.structured_issues.len(), 1);
        assert_eq!(report.structured_issues[0].code, PackageValidationIssueCode::MissingFixture);
        assert_eq!(report.structured_issues[0].field, "fixtures");
    }

    #[test]
    fn validate_package_manifest_rejects_fixture_with_missing_fields() {
        let mut manifest = minimal_manifest();
        manifest.fixtures = vec![PackageFixture {
            name: "".into(),
            input: "".into(),
            expected_output: "".into(),
            input_hash: None,
            expected_output_hash: None,
        }];
        let report = validate_package_manifest(&manifest);
        assert!(!report.valid);
        assert_eq!(report.structured_issues.len(), 3);
        let fields: Vec<&str> = report
            .structured_issues
            .iter()
            .map(|i| i.field.as_str())
            .collect();
        assert!(fields.contains(&"fixtures[0].name"));
        assert!(fields.contains(&"fixtures[0].input"));
        assert!(fields.contains(&"fixtures[0].expected_output"));
    }

    #[test]
    fn validate_package_manifest_rejects_empty_shape() {
        let mut manifest = minimal_manifest();
        manifest.expected_input = Some(DataShape {
            shape: vec![],
            dtype: DataType::Float32,
        });
        manifest.expected_output = Some(DataShape {
            shape: vec![],
            dtype: DataType::Uint8,
        });
        let report = validate_package_manifest(&manifest);
        assert!(!report.valid);
        assert_eq!(report.structured_issues.len(), 2);
        let fields: Vec<&str> = report
            .structured_issues
            .iter()
            .map(|i| i.field.as_str())
            .collect();
        assert!(fields.contains(&"expected_input.shape"));
        assert!(fields.contains(&"expected_output.shape"));
    }

    #[test]
    fn validate_package_manifest_accepts_non_empty_shape() {
        let mut manifest = minimal_manifest();
        manifest.expected_input = Some(DataShape {
            shape: vec!["1".into(), "256".into()],
            dtype: DataType::Float32,
        });
        let report = validate_package_manifest(&manifest);
        assert!(report.valid);
    }
}

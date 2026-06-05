use biors_core::package::{validate_package_manifest, DataShape, DataType};

mod common;

#[test]
fn validate_package_manifest_rejects_empty_shape() {
    let mut manifest = common::valid_manifest();
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
fn validate_package_manifest_rejects_empty_shape_dimensions() {
    let mut manifest = common::valid_manifest();
    manifest.expected_input = Some(DataShape {
        shape: vec!["".into(), "256".into()],
        dtype: DataType::Float32,
    });
    manifest.expected_output = Some(DataShape {
        shape: vec![" ".into()],
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
    assert!(fields.contains(&"expected_input.shape[0]"));
    assert!(fields.contains(&"expected_output.shape[0]"));
}

#[test]
fn validate_package_manifest_accepts_non_empty_shape() {
    let mut manifest = common::valid_manifest();
    manifest.expected_input = Some(DataShape {
        shape: vec!["batch".into(), "256".into(), "features".into()],
        dtype: DataType::Float32,
    });
    let report = validate_package_manifest(&manifest);
    assert!(report.valid);
}

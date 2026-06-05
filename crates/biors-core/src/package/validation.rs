mod contracts;
mod fixtures;
mod required;
mod shapes;

use super::{PackageManifest, PackageValidationReport};
use contracts::{validate_contract_identifiers, validate_runtime_contract, validate_v1_contract};
use fixtures::validate_fixture_list;
use required::push_required_issue;
use shapes::validate_shape;

/// Validate package manifest fields that do not require filesystem access.
pub fn validate_package_manifest(manifest: &PackageManifest) -> PackageValidationReport {
    let mut report = PackageValidationReport::default();

    push_required_issue(&mut report, "name", &manifest.name);
    validate_v1_contract(&mut report, manifest);
    push_required_issue(&mut report, "model.path", &manifest.model.path);
    if let Some(metadata) = &manifest.model.metadata {
        push_required_issue(&mut report, "model.metadata.name", &metadata.name);
    }
    validate_contract_identifiers(&mut report, manifest);
    validate_runtime_contract(&mut report, manifest);
    validate_fixture_list(&mut report, manifest);
    for (field, shape) in [
        ("expected_input", manifest.expected_input.as_ref()),
        ("expected_output", manifest.expected_output.as_ref()),
    ] {
        if let Some(shape) = shape {
            validate_shape(&mut report, field, shape);
        }
    }

    report.finish()
}

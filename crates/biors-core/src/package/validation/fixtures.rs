use super::super::{PackageManifest, PackageValidationIssueCode, PackageValidationReport};
use super::required::push_required_issue;
use std::collections::HashMap;

pub(super) fn validate_fixture_list(
    report: &mut PackageValidationReport,
    manifest: &PackageManifest,
) {
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
    reject_duplicate_fixture_names(report, manifest);
}

fn reject_duplicate_fixture_names(
    report: &mut PackageValidationReport,
    manifest: &PackageManifest,
) {
    let mut first_index_by_name: HashMap<&str, usize> = HashMap::new();
    for (index, fixture) in manifest.fixtures.iter().enumerate() {
        let name = fixture.name.trim();
        if name.is_empty() {
            continue;
        }
        match first_index_by_name.get(name) {
            Some(first_index) => report.push_issue(
                PackageValidationIssueCode::DuplicateFixtureName,
                &format!("fixtures[{index}].name"),
                &format!("fixture name '{name}' duplicates fixtures[{first_index}].name"),
            ),
            None => {
                first_index_by_name.insert(name, index);
            }
        }
    }
}

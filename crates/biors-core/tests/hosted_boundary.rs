use biors_core::service::current_hosted_workflow_boundary;

#[test]
fn hosted_boundary_preserves_local_first_core_contract() {
    let boundary = current_hosted_workflow_boundary();

    assert_eq!(boundary.schema_version, "biors.hosted_workflow_boundary.v0");
    assert_eq!(boundary.product_name, "bio-rs");
    assert_eq!(boundary.status, "boundary_contract_only");
    assert!(boundary.execution_policy.local_first);
    assert!(boundary.execution_policy.no_network_by_default);
    assert!(!boundary.execution_policy.biological_data_uploads_by_default);
    assert!(!boundary.execution_policy.external_model_calls_by_default);
    assert!(!boundary.execution_policy.telemetry_by_default);
    assert!(
        boundary
            .execution_policy
            .remote_processing_requires_explicit_consent
    );
}

#[test]
fn hosted_boundary_keeps_workspace_and_billing_outside_core() {
    let boundary = current_hosted_workflow_boundary();

    assert_contains(&boundary.open_source_core.excluded, "hosted user accounts");
    assert_contains(
        &boundary.open_source_core.excluded,
        "hosted project workspaces",
    );
    assert_contains(
        &boundary.open_source_core.excluded,
        "paid billing and subscription logic",
    );
    assert_contains(
        &boundary.hosted_layer.allowed,
        "project workspace persistence",
    );
    assert_contains(
        &boundary.hosted_layer.allowed,
        "billing and subscription management",
    );
    assert!(!boundary.commercial_policy.billing_in_core);
    assert!(
        boundary
            .commercial_policy
            .must_remain_separate_from_open_source_core
    );
    assert!(
        !boundary
            .commercial_policy
            .core_package_behavior_changes_for_hosted_service
    );
}

#[test]
fn hosted_boundary_models_research_workspaces_without_implementing_them_in_core() {
    let boundary = current_hosted_workflow_boundary();
    let workspace_names: Vec<_> = boundary
        .workspace_model
        .iter()
        .map(|workspace| workspace.name.as_str())
        .collect();

    assert!(workspace_names.contains(&"UserWorkspace"));
    assert!(workspace_names.contains(&"ProjectWorkspace"));
    assert!(workspace_names.contains(&"WorkflowRun"));
    assert!(boundary
        .workspace_model
        .iter()
        .all(|workspace| !workspace.implemented_in_core));
    assert!(!boundary.web_product_policy.product_web_runtime_in_core);
    assert!(
        !boundary
            .web_product_policy
            .product_landing_page_in_repository
    );
    assert_eq!(boundary.web_product_policy.launch_track, "1.0");
    assert_contains(
        &boundary.validation_requirements,
        "no silent upload of biological data",
    );
    assert_contains(
        &boundary.validation_requirements,
        "version-pinned bio-rs package and schema identifiers in every hosted result",
    );
}

fn assert_contains(values: &[String], expected: &str) {
    assert!(
        values.iter().any(|value| value == expected),
        "missing {expected:?} in {values:?}"
    );
}

use serde::{Deserialize, Serialize};

pub const HOSTED_WORKFLOW_BOUNDARY_SCHEMA_VERSION: &str = "biors.hosted_workflow_boundary.v0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedWorkflowBoundary {
    pub schema_version: String,
    pub product_name: String,
    pub core_version: String,
    pub status: String,
    pub execution_policy: HostedExecutionPolicy,
    pub open_source_core: HostedResponsibilitySet,
    pub hosted_layer: HostedResponsibilitySet,
    pub workspace_model: Vec<HostedWorkspaceConcept>,
    pub commercial_policy: HostedCommercialPolicy,
    pub web_product_policy: HostedWebProductPolicy,
    pub validation_requirements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedExecutionPolicy {
    pub local_first: bool,
    pub no_network_by_default: bool,
    pub biological_data_uploads_by_default: bool,
    pub external_model_calls_by_default: bool,
    pub telemetry_by_default: bool,
    pub remote_processing_requires_explicit_consent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedResponsibilitySet {
    pub owner: String,
    pub allowed: Vec<String>,
    pub excluded: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedWorkspaceConcept {
    pub name: String,
    pub status: String,
    pub implemented_in_core: bool,
    pub owner: String,
    pub data_classification: String,
    pub required_controls: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedCommercialPolicy {
    pub paid_hosted_service_allowed: bool,
    pub billing_in_core: bool,
    pub must_remain_separate_from_open_source_core: bool,
    pub core_package_behavior_changes_for_hosted_service: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedWebProductPolicy {
    pub product_web_runtime_in_core: bool,
    pub product_landing_page_in_repository: bool,
    pub launch_track: String,
    pub notes: Vec<String>,
}

pub fn current_hosted_workflow_boundary() -> HostedWorkflowBoundary {
    hosted_workflow_boundary(env!("CARGO_PKG_VERSION"))
}

pub fn hosted_workflow_boundary(version: impl Into<String>) -> HostedWorkflowBoundary {
    HostedWorkflowBoundary {
        schema_version: HOSTED_WORKFLOW_BOUNDARY_SCHEMA_VERSION.to_string(),
        product_name: "bio-rs".to_string(),
        core_version: version.into(),
        status: "boundary_contract_only".to_string(),
        execution_policy: HostedExecutionPolicy {
            local_first: true,
            no_network_by_default: true,
            biological_data_uploads_by_default: false,
            external_model_calls_by_default: false,
            telemetry_by_default: false,
            remote_processing_requires_explicit_consent: true,
        },
        open_source_core: HostedResponsibilitySet {
            owner: "open_source_core".to_string(),
            allowed: vec![
                "deterministic local validation".to_string(),
                "deterministic local tokenization".to_string(),
                "format, structure, molecule, conversion, report, and template contracts"
                    .to_string(),
                "local CLI execution".to_string(),
                "local WASM package APIs".to_string(),
                "local service contracts and local HTTP mode".to_string(),
                "schema and provenance contracts for externally hosted callers".to_string(),
            ],
            excluded: vec![
                "hosted user accounts".to_string(),
                "hosted project workspaces".to_string(),
                "hosted workflow storage".to_string(),
                "paid billing and subscription logic".to_string(),
                "hosted product landing page runtime".to_string(),
                "silent biological data upload".to_string(),
                "remote model execution by default".to_string(),
                "telemetry or analytics by default".to_string(),
            ],
        },
        hosted_layer: HostedResponsibilitySet {
            owner: "separate_hosted_web_or_service_layer".to_string(),
            allowed: vec![
                "user identity and organization membership".to_string(),
                "project workspace persistence".to_string(),
                "remote object storage with explicit consent".to_string(),
                "workflow run history".to_string(),
                "billing and subscription management".to_string(),
                "audit logging and operational monitoring".to_string(),
                "product web UI and landing pages".to_string(),
            ],
            excluded: vec![
                "changing open-source core validation semantics".to_string(),
                "claiming hosted results without pinned bio-rs version and schema".to_string(),
                "uploading biological data without explicit consent".to_string(),
                "retaining user data without retention, deletion, and export controls"
                    .to_string(),
            ],
        },
        workspace_model: vec![
            workspace_concept(
                "UserWorkspace",
                "account and membership boundary",
                "identity, billing, preferences, and consent records",
                [
                    "authentication",
                    "authorization",
                    "consent audit trail",
                    "data export",
                    "account deletion",
                ],
            ),
            workspace_concept(
                "ProjectWorkspace",
                "research project boundary",
                "project metadata, package references, reports, and run indexes",
                [
                    "member isolation",
                    "project-level retention",
                    "provenance preservation",
                    "share/export controls",
                ],
            ),
            workspace_concept(
                "WorkflowRun",
                "single execution boundary",
                "input references, validation output, tokenization output, reports, and logs",
                [
                    "version-pinned schemas",
                    "input/output hash provenance",
                    "repeatable report export",
                    "explicit remote execution consent",
                ],
            ),
        ],
        commercial_policy: HostedCommercialPolicy {
            paid_hosted_service_allowed: true,
            billing_in_core: false,
            must_remain_separate_from_open_source_core: true,
            core_package_behavior_changes_for_hosted_service: false,
            notes: vec![
                "Published Rust, Python, WASM, MCP, and CLI packages remain local-first."
                    .to_string(),
                "Hosted billing must not gate local validation, tokenization, conversion, or reporting behavior."
                    .to_string(),
            ],
        },
        web_product_policy: HostedWebProductPolicy {
            product_web_runtime_in_core: false,
            product_landing_page_in_repository: false,
            launch_track: "1.0".to_string(),
            notes: vec![
                "The repository does not ship a hosted product UI in this contract."
                    .to_string(),
                "Product web and landing-page code belongs to a separate hosted layer."
                    .to_string(),
            ],
        },
        validation_requirements: vec![
            "no silent upload of biological data".to_string(),
            "explicit consent before remote processing".to_string(),
            "project and organization isolation".to_string(),
            "retention, deletion, and export controls for persisted hosted data".to_string(),
            "version-pinned bio-rs package and schema identifiers in every hosted result"
                .to_string(),
            "input and output hashes recorded for reproducibility".to_string(),
            "human-readable report export with provenance".to_string(),
            "auditable authorization and billing events outside the core runtime".to_string(),
        ],
    }
}

fn workspace_concept(
    name: &str,
    status: &str,
    data_classification: &str,
    required_controls: impl IntoIterator<Item = &'static str>,
) -> HostedWorkspaceConcept {
    HostedWorkspaceConcept {
        name: name.to_string(),
        status: status.to_string(),
        implemented_in_core: false,
        owner: "separate_hosted_layer".to_string(),
        data_classification: data_classification.to_string(),
        required_controls: required_controls.into_iter().map(str::to_string).collect(),
    }
}

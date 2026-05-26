//! Transport-agnostic service interface contracts.
//!
//! This module describes the stable JSON boundary for embedding bio-rs behind
//! a service wrapper. It intentionally does not start a network listener or own
//! HTTP runtime behavior; callers can adapt these contracts to their service
//! stack while keeping core bio-rs behavior deterministic.

use serde::{Deserialize, Serialize};

pub const SERVICE_INTERFACE_SCHEMA_VERSION: &str = "biors.service_interface.v0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceInterfaceDocument {
    pub schema_version: String,
    pub service_name: String,
    pub service_version: String,
    pub server_runtime: String,
    pub transport_model: String,
    pub runtime_separation: RuntimeServiceSeparation,
    pub openapi: OpenApiDirection,
    pub routes: Vec<ServiceRoute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeServiceSeparation {
    pub core_contract_owner: String,
    pub service_runtime_owner: String,
    pub permitted_in_core: Vec<String>,
    pub forbidden_in_core: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenApiDirection {
    pub status: String,
    pub title: String,
    pub version: String,
    pub schema_base_uri: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceRoute {
    pub operation_id: String,
    pub domain: String,
    pub method: String,
    pub path: String,
    pub request_schema: String,
    pub response_schema: String,
    pub deterministic: bool,
    pub idempotent: bool,
    pub file_access: String,
    pub runtime_boundary: String,
}

pub fn current_service_interface_document() -> ServiceInterfaceDocument {
    service_interface_document(env!("CARGO_PKG_VERSION"))
}

pub fn service_interface_document(version: impl Into<String>) -> ServiceInterfaceDocument {
    let version = version.into();
    ServiceInterfaceDocument {
        schema_version: SERVICE_INTERFACE_SCHEMA_VERSION.to_string(),
        service_name: "bio-rs".to_string(),
        service_version: version.clone(),
        server_runtime: "not_included".to_string(),
        transport_model: "transport_agnostic_json_contract".to_string(),
        runtime_separation: RuntimeServiceSeparation {
            core_contract_owner: "biors-core".to_string(),
            service_runtime_owner: "external service host".to_string(),
            permitted_in_core: vec![
                "deterministic sequence parsing".to_string(),
                "tokenization and model input construction".to_string(),
                "read-only package inspection and compatibility planning".to_string(),
                "runtime bridge planning without backend execution".to_string(),
            ],
            forbidden_in_core: vec![
                "network listener".to_string(),
                "request authentication".to_string(),
                "rate limiting".to_string(),
                "background job queue".to_string(),
                "remote object storage access".to_string(),
            ],
        },
        openapi: OpenApiDirection {
            status: "offline_contract".to_string(),
            title: "bio-rs service interface".to_string(),
            version,
            schema_base_uri: "https://bio-rs.dev/schemas/".to_string(),
            notes: vec![
                "OpenAPI generation is a host responsibility, derived from these stable operation contracts.".to_string(),
                "The core crate does not expose an HTTP server or bind to a socket.".to_string(),
            ],
        },
        routes: service_routes(),
    }
}

pub fn service_routes() -> Vec<ServiceRoute> {
    vec![
        route(
            "sequence.validate",
            "sequence",
            "/v0/sequence/validate",
            "service-sequence-validate-request.v0.json",
            "fasta-validation-output.v0.json",
            "read_only_input",
            "core_deterministic",
        ),
        route(
            "sequence.inspect",
            "sequence",
            "/v0/sequence/inspect",
            "service-sequence-inspect-request.v0.json",
            "inspect-output.v0.json",
            "read_only_input",
            "core_deterministic",
        ),
        route(
            "sequence.tokenize",
            "tokenizer",
            "/v0/sequence/tokenize",
            "service-sequence-tokenize-request.v0.json",
            "tokenize-output.v0.json",
            "read_only_input",
            "core_deterministic",
        ),
        route(
            "model_input.build",
            "model_input",
            "/v0/model-input/build",
            "service-model-input-request.v0.json",
            "model-input-output.v0.json",
            "read_only_input",
            "core_deterministic",
        ),
        route(
            "package.inspect",
            "package",
            "/v0/package/inspect",
            "service-package-request.v0.json",
            "package-inspect-output.v0.json",
            "package_read_only",
            "package_contract",
        ),
        route(
            "package.validate",
            "package",
            "/v0/package/validate",
            "service-package-request.v0.json",
            "package-validation-report.v0.json",
            "package_read_only",
            "package_contract",
        ),
        route(
            "package.bridge.plan",
            "runtime",
            "/v0/package/bridge/plan",
            "service-package-request.v0.json",
            "package-bridge-output.v0.json",
            "package_read_only",
            "runtime_planning_only",
        ),
        route(
            "package.compatibility.compare",
            "package",
            "/v0/package/compatibility/compare",
            "service-package-compatibility-request.v0.json",
            "package-compatibility-output.v0.json",
            "package_read_only",
            "package_contract",
        ),
    ]
}

fn route(
    operation_id: &str,
    domain: &str,
    path: &str,
    request_schema: &str,
    response_schema: &str,
    file_access: &str,
    runtime_boundary: &str,
) -> ServiceRoute {
    ServiceRoute {
        operation_id: operation_id.to_string(),
        domain: domain.to_string(),
        method: "POST".to_string(),
        path: path.to_string(),
        request_schema: request_schema.to_string(),
        response_schema: response_schema.to_string(),
        deterministic: true,
        idempotent: true,
        file_access: file_access.to_string(),
        runtime_boundary: runtime_boundary.to_string(),
    }
}

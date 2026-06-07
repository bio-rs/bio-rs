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
        server_runtime: "cli_local_http_server".to_string(),
        transport_model: "local_first_http_json".to_string(),
        runtime_separation: RuntimeServiceSeparation {
            core_contract_owner: "biors-core".to_string(),
            service_runtime_owner: "biors CLI local server".to_string(),
            permitted_in_core: vec![
                "deterministic inline FASTA batch validation".to_string(),
                "service route, schema, health, and OpenAPI metadata for the local runtime"
                    .to_string(),
                "local-first request and response contracts for served endpoints".to_string(),
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
            status: "served_by_cli_runtime".to_string(),
            title: "bio-rs service interface".to_string(),
            version,
            schema_base_uri: "https://bio-rs.dev/schemas/".to_string(),
            notes: vec![
                "biors serve exposes GET /openapi.json for the local HTTP runtime.".to_string(),
                "biors-core still does not bind sockets or own service runtime behavior."
                    .to_string(),
            ],
        },
        routes: service_routes(),
    }
}

pub fn service_routes() -> Vec<ServiceRoute> {
    vec![
        route(
            "service.health",
            "service",
            "GET",
            "/health",
            (
                "service-empty-request.v0.json",
                "service-health-output.v0.json",
            ),
            "no_file_access",
            "cli_local_server",
        ),
        route(
            "service.openapi",
            "service",
            "GET",
            "/openapi.json",
            (
                "service-empty-request.v0.json",
                "service-openapi-output.v0.json",
            ),
            "no_file_access",
            "cli_local_server",
        ),
        route(
            "sequence.batch_validate",
            "sequence",
            "POST",
            "/v0/batch/sequence/validate",
            (
                "service-batch-sequence-validate-request.v0.json",
                "service-batch-sequence-validate-output.v0.json",
            ),
            "read_only_input",
            "core_deterministic",
        ),
    ]
}

pub fn local_service_routes() -> Vec<ServiceRoute> {
    service_routes()
}

fn route(
    operation_id: &str,
    domain: &str,
    method: &str,
    path: &str,
    schemas: (&str, &str),
    file_access: &str,
    runtime_boundary: &str,
) -> ServiceRoute {
    ServiceRoute {
        operation_id: operation_id.to_string(),
        domain: domain.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        request_schema: schemas.0.to_string(),
        response_schema: schemas.1.to_string(),
        deterministic: true,
        idempotent: true,
        file_access: file_access.to_string(),
        runtime_boundary: runtime_boundary.to_string(),
    }
}

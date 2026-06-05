use super::local_service_routes;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

pub const SERVICE_HEALTH_SCHEMA_VERSION: &str = "biors.service_health.v0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceHealthDocument {
    pub schema_version: String,
    pub service_name: String,
    pub service_version: String,
    pub status: String,
    pub network_policy: String,
    pub endpoints: Vec<ServiceEndpointStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceEndpointStatus {
    pub method: String,
    pub path: String,
    pub operation_id: String,
}

pub fn service_health_document(version: impl Into<String>) -> ServiceHealthDocument {
    ServiceHealthDocument {
        schema_version: SERVICE_HEALTH_SCHEMA_VERSION.to_string(),
        service_name: "bio-rs".to_string(),
        service_version: version.into(),
        status: "ok".to_string(),
        network_policy: "local_first_no_external_calls".to_string(),
        endpoints: local_service_routes()
            .into_iter()
            .map(|route| ServiceEndpointStatus {
                method: route.method,
                path: route.path,
                operation_id: route.operation_id,
            })
            .collect(),
    }
}

pub fn service_openapi_document(
    version: impl Into<String>,
    server_url: impl Into<String>,
) -> Value {
    let version = version.into();
    let server_url = server_url.into();
    let mut paths = Map::new();

    for route in local_service_routes() {
        let method = route.method.to_lowercase();
        let path_item = paths.entry(route.path.clone()).or_insert_with(|| json!({}));
        let path_object = path_item
            .as_object_mut()
            .expect("OpenAPI path items are objects");
        path_object.insert(method, openapi_operation(&route));
    }

    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "bio-rs local service API",
            "version": version,
            "description": "Local-first HTTP API for deterministic bio-rs validation and preprocessing."
        },
        "servers": [{ "url": server_url }],
        "paths": paths,
        "components": { "schemas": {} }
    })
}

fn openapi_operation(route: &super::ServiceRoute) -> Value {
    let response_ref = schema_ref(&route.response_schema);
    let mut operation = json!({
        "operationId": route.operation_id,
        "summary": route.operation_id,
        "responses": {
            "200": {
                "description": "Successful response",
                "content": {
                    "application/json": {
                        "schema": { "$ref": response_ref }
                    }
                }
            },
            "400": { "description": "Invalid request" },
            "404": { "description": "Route not found" },
            "405": { "description": "Method not allowed" },
            "413": { "description": "Request body too large" },
            "422": { "description": "Request payload failed biological validation" }
        }
    });

    if route.method == "POST" {
        operation["requestBody"] = json!({
            "required": true,
            "content": {
                "application/json": {
                    "schema": { "$ref": schema_ref(&route.request_schema) }
                }
            }
        });
    }

    operation
}

fn schema_ref(name: &str) -> String {
    format!("https://bio-rs.dev/schemas/{name}")
}

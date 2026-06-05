use super::serve_http::{HttpRequest, HttpResponse};
use biors_core::service::{
    service_health_document, service_openapi_document, validate_service_batch_sequence_request,
    ServiceBatchSequenceValidateRequest,
};

pub(crate) fn handle_service_request(
    request: HttpRequest,
    version: &str,
    base_url: &str,
) -> HttpResponse {
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/health") => HttpResponse::json(200, &service_health_document(version)),
        ("GET", "/openapi.json") => {
            HttpResponse::json(200, &service_openapi_document(version, base_url))
        }
        ("POST", "/v0/batch/sequence/validate") => handle_batch_sequence_validate(request),
        (method, path) if known_path(path) => HttpResponse::error(
            405,
            "service.method_not_allowed",
            format!("{method} is not allowed for {path}"),
            Some(path.to_string()),
        ),
        (_, path) => HttpResponse::error(
            404,
            "service.route_not_found",
            format!("service route '{path}' was not found"),
            Some(path.to_string()),
        ),
    }
}

fn handle_batch_sequence_validate(request: HttpRequest) -> HttpResponse {
    let payload: ServiceBatchSequenceValidateRequest = match serde_json::from_slice(&request.body) {
        Ok(payload) => payload,
        Err(error) => {
            return HttpResponse::error(
                400,
                "service.invalid_json",
                format!("request body is not valid JSON: {error}"),
                None,
            )
        }
    };

    match validate_service_batch_sequence_request(payload) {
        Ok(output) => HttpResponse::json(200, &output),
        Err(error) => HttpResponse::error(422, error.code(), error.to_string(), error.location()),
    }
}

fn known_path(path: &str) -> bool {
    matches!(
        path,
        "/health" | "/openapi.json" | "/v0/batch/sequence/validate"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn health_returns_local_policy() {
        let response = handle_service_request(
            HttpRequest::get("/health"),
            "0.54.0",
            "http://127.0.0.1:8787",
        );
        assert_eq!(response.status, 200);
        let body: Value = serde_json::from_slice(&response.body).expect("health JSON");
        assert_eq!(body["schema_version"], "biors.service_health.v0");
        assert_eq!(body["network_policy"], "local_first_no_external_calls");
    }

    #[test]
    fn batch_sequence_validate_accepts_inline_fasta() {
        let request = HttpRequest::post(
            "/v0/batch/sequence/validate",
            br#"{"kind":"protein","inputs":[{"id":"seqs","fasta_text":">seq1\nACDE\n"}]}"#.to_vec(),
        );
        let response = handle_service_request(request, "0.54.0", "http://127.0.0.1:8787");
        assert_eq!(response.status, 200);
        let body: Value = serde_json::from_slice(&response.body).expect("batch JSON");
        assert_eq!(
            body["schema_version"],
            "biors.service_batch_sequence_validate.v0"
        );
        assert_eq!(body["summary"]["fasta_records"], 1);
    }

    #[test]
    fn batch_sequence_validate_reports_payload_errors() {
        let request = HttpRequest::post(
            "/v0/batch/sequence/validate",
            br#"{"inputs":[{"id":"","fasta_text":">seq1\nACDE\n"}]}"#.to_vec(),
        );
        let response = handle_service_request(request, "0.54.0", "http://127.0.0.1:8787");
        assert_eq!(response.status, 422);
        let body: Value = serde_json::from_slice(&response.body).expect("error JSON");
        assert_eq!(body["error"]["code"], "service.batch.empty_input_id");
    }

    #[test]
    fn batch_sequence_validate_rejects_unknown_fields_and_invalid_ids() {
        let unknown_field = HttpRequest::post(
            "/v0/batch/sequence/validate",
            br#"{"inputs":[{"id":"seq1","fasta_text":">seq1\nACDE\n","extra":true}]}"#.to_vec(),
        );
        let response = handle_service_request(unknown_field, "0.57.0", "http://127.0.0.1:8787");
        assert_eq!(response.status, 400);
        let body: Value = serde_json::from_slice(&response.body).expect("error JSON");
        assert_eq!(body["error"]["code"], "service.invalid_json");

        let invalid_id = HttpRequest::post(
            "/v0/batch/sequence/validate",
            br#"{"inputs":[{"id":"sample 1","fasta_text":">seq1\nACDE\n"}]}"#.to_vec(),
        );
        let response = handle_service_request(invalid_id, "0.57.0", "http://127.0.0.1:8787");
        assert_eq!(response.status, 422);
        let body: Value = serde_json::from_slice(&response.body).expect("error JSON");
        assert_eq!(body["error"]["code"], "service.batch.invalid_input_id");
    }
}

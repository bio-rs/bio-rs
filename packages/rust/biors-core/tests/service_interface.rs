use biors_core::service::current_service_interface_document;

#[test]
fn service_interface_documents_transport_agnostic_boundary() {
    let document = current_service_interface_document();

    assert_eq!(document.schema_version, "biors.service_interface.v0");
    assert_eq!(document.service_name, "bio-rs");
    assert_eq!(document.server_runtime, "not_included");
    assert_eq!(document.openapi.status, "offline_contract");
    assert!(document
        .runtime_separation
        .forbidden_in_core
        .contains(&"network listener".to_string()));
}

#[test]
fn service_interface_lists_research_workflow_operations() {
    let document = current_service_interface_document();
    let operations: Vec<_> = document
        .routes
        .iter()
        .map(|route| route.operation_id.as_str())
        .collect();

    assert!(operations.contains(&"sequence.validate"));
    assert!(operations.contains(&"sequence.tokenize"));
    assert!(operations.contains(&"model_input.build"));
    assert!(operations.contains(&"package.inspect"));
    assert!(operations.contains(&"package.validate"));
    assert!(operations.contains(&"package.bridge.plan"));
    assert!(operations.contains(&"package.compatibility.compare"));
    assert!(!operations
        .iter()
        .any(|operation| operation.contains("serve")));
}

#[test]
fn service_routes_have_stable_json_contracts() {
    let document = current_service_interface_document();

    for route in document.routes {
        assert!(route.path.starts_with("/v0/"), "{route:?}");
        assert!(route.request_schema.ends_with(".v0.json"), "{route:?}");
        assert!(route.response_schema.ends_with(".v0.json"), "{route:?}");
        assert!(route.deterministic, "{route:?}");
        assert_ne!(route.runtime_boundary, "http_server_runtime");
    }
}

use std::fs;

mod common;

#[test]
fn wasm_and_service_docs_keep_integration_boundaries_explicit() {
    let repo = common::repo_root();
    let stability = fs::read_to_string(repo.join("docs/1-0-stability.md")).expect("stability doc");
    let wasm = fs::read_to_string(repo.join("docs/wasm-api.md")).expect("WASM docs");
    let service = fs::read_to_string(repo.join("docs/service-interface.md")).expect("service docs");
    let docs = normalize_whitespace(&format!("{stability}\n{wasm}\n{service}"));

    for required in [
        "Local browser/JavaScript validation and workflow embedding, not browser model execution.",
        "No hosted service or external runtime is required.",
        "no network access",
        "no `fetch` calls",
        "no external model calls",
        "Tokenization currently supports FASTA only.",
        "The default bind address is `127.0.0.1:8787`.",
        "no external network calls, uploads, telemetry, model inference",
        "hosted or production layer is caller-owned",
    ] {
        assert!(
            docs.contains(required),
            "integration docs missing boundary: {required}"
        );
    }
}

#[test]
fn service_tests_cover_only_documented_local_routes() {
    let repo = common::repo_root();
    let tests =
        fs::read_to_string(repo.join("crates/biors/tests/cli_service.rs")).expect("service tests");
    let service_doc =
        fs::read_to_string(repo.join("docs/service-interface.md")).expect("service docs");

    for route in ["/health", "/openapi.json", "/v0/batch/sequence/validate"] {
        assert!(tests.contains(route), "service tests missing {route}");
        assert!(service_doc.contains(route), "service docs missing {route}");
    }
    assert!(tests.contains("/v0/package/bridge/plan"));
    assert!(tests.contains("service.route_not_found"));
    assert!(tests.contains("service.method_not_allowed"));
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

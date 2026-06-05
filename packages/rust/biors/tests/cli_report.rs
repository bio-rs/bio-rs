use std::fs;

use serde_json::Value;

mod common;

use common::TempDir;

#[test]
fn report_generate_exports_shareable_json_and_markdown() {
    let temp = TempDir::new("biors-report");
    let input = temp.write(
        "workflow.json",
        r#"{
          "ok": true,
          "biors_version": "0.55.0",
          "input_hash": "fnv1a64:08a331cb13c7bd72",
          "data": {
            "workflow": "protein_model_input.v0",
            "model_ready": true,
            "provenance": {},
            "validation": {
              "valid": true,
              "records": 1,
              "valid_records": 1,
              "warning_count": 0,
              "error_count": 0
            },
            "tokenization": {
              "summary": {
                "records": 1,
                "tokens": 4
              },
              "records": []
            },
            "readiness_issues": []
          }
        }"#,
    );
    let markdown = temp.path().join("report.md");
    let shareable = temp.path().join("report.json");

    let output = common::run_biors(&[
        "report",
        "generate",
        input.to_str().expect("input path"),
        "--output",
        markdown.to_str().expect("markdown path"),
        "--shareable-json",
        shareable.to_str().expect("shareable path"),
    ]);
    common::assert_payload_matches_schema(&output.stdout, "schemas/report-output.v0.json");

    let envelope: Value = serde_json::from_slice(&output.stdout).expect("valid CLI JSON");
    assert_eq!(envelope["data"]["schema_version"], "biors.report.v0");
    assert_eq!(envelope["data"]["status"], "pass");
    assert_eq!(
        envelope["data"]["provenance"]["source_input_hash"],
        "fnv1a64:08a331cb13c7bd72"
    );

    let markdown = fs::read_to_string(markdown).expect("read markdown report");
    assert!(markdown.contains("# bio-rs Workflow Report"));
    assert!(markdown.contains("input raw SHA-256"));

    let shareable_json = fs::read(&shareable).expect("read shareable JSON");
    common::assert_json_matches_schema(&shareable_json, "schemas/report-output.v0.json");
}

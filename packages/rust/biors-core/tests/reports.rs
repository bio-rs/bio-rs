use biors_core::reports::{
    build_shareable_report_from_json_bytes, ReportInputContainer, ReportInputKind, ReportStatus,
    REPORT_SCHEMA_VERSION,
};

#[test]
fn builds_reproducible_report_from_workflow_cli_envelope() {
    let input = br#"{
      "ok": true,
      "biors_version": "0.55.0",
      "input_hash": "fnv1a64:08a331cb13c7bd72",
      "data": {
        "workflow": "protein_model_input.v0",
        "model_ready": false,
        "provenance": {},
        "validation": {
          "valid": false,
          "records": 1,
          "valid_records": 0,
          "warning_count": 0,
          "error_count": 1
        },
        "tokenization": {
          "summary": {
            "records": 1,
            "tokens": 4
          },
          "records": []
        },
        "readiness_issues": [{
          "id": "seq1",
          "message": "sequence 'seq1' is not model-ready"
        }]
      }
    }"#;

    let report = build_shareable_report_from_json_bytes(input).expect("build report");
    let same = build_shareable_report_from_json_bytes(input).expect("build second report");

    assert_eq!(report, same);
    assert_eq!(report.schema_version, REPORT_SCHEMA_VERSION);
    assert_eq!(report.status, ReportStatus::Fail);
    assert_eq!(
        report.provenance.input_container,
        ReportInputContainer::CliSuccessEnvelope
    );
    assert_eq!(
        report.provenance.input_kind,
        ReportInputKind::SequenceWorkflowOutput
    );
    assert_eq!(
        report.provenance.source_input_hash.as_deref(),
        Some("fnv1a64:08a331cb13c7bd72")
    );
    assert!(report.human_report.contains("# bio-rs Workflow Report"));
    assert!(report
        .human_report
        .contains("sequence 'seq1' is not model-ready"));
}

#[test]
fn conversion_reports_include_counts_and_status() {
    let input = br#"{
      "schema_version": "biors.conversion.v0",
      "records": 2,
      "valid_records": 1,
      "model_ready_records": 1,
      "warning_count": 1,
      "error_count": 1,
      "entities": [{
        "id": "seq1",
        "entity_type": "sequence",
        "source": { "format": "fasta" },
        "record": { "type": "sequence", "data": {} },
        "validation": {
          "valid": false,
          "model_ready": false,
          "warning_count": 0,
          "error_count": 1,
          "warnings": [],
          "errors": [{
            "severity": "error",
            "code": "sequence_invalid_symbol",
            "message": "invalid residue"
          }]
        }
      }]
    }"#;

    let report = build_shareable_report_from_json_bytes(input).expect("build report");

    assert_eq!(report.status, ReportStatus::Fail);
    assert_eq!(
        report.provenance.input_kind,
        ReportInputKind::BioEntityExport
    );
    assert!(report
        .sections
        .iter()
        .any(|section| section.id == "record_counts"));
    assert!(report.human_report.contains("Converted 2 records"));
}

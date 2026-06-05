use serde_json::Value;

mod cli_pipeline_support;
mod common;

use cli_pipeline_support::run_biors;
use common::TempDir;

#[test]
fn diff_reports_canonical_json_matches_and_mismatches() {
    let temp = TempDir::new("biors-diff");
    let expected = temp.write("expected.json", r#"{"tokens":[1,2],"id":"seq1"}"#);
    let reordered = temp.write("reordered.json", r#"{"id":"seq1","tokens":[1,2]}"#);
    let mismatch = temp.write("mismatch.json", r#"{"id":"seq1","tokens":[1,3]}"#);

    let matching = run_biors(&["diff"], &[&expected, &reordered]);
    assert_eq!(matching["data"]["matches"], true);
    assert!(matching["data"]["expected_sha256"]
        .as_str()
        .expect("expected hash")
        .starts_with("sha256:"));
    assert_eq!(matching["data"]["content_diff"], Value::Null);

    let different = run_biors(&["diff"], &[&expected, &mismatch]);
    assert_eq!(different["data"]["matches"], false);
    assert_ne!(
        different["data"]["expected_sha256"],
        different["data"]["observed_sha256"]
    );
    assert_eq!(
        different["data"]["content_diff"]["expected_path"],
        expected.display().to_string()
    );
    assert!(
        different["data"]["content_diff"]["first_difference"]["byte_offset"]
            .as_u64()
            .is_some()
    );
}

use serde_json::Value;

mod common;

use common::ChildInputExt;

#[test]
fn debug_outputs_step_by_step_tokens_model_input_and_error_visualization() {
    let output = common::spawn_biors(&["debug", "--max-length", "6", "-"]).tap_stdin(">bad\nAX*\n");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    let record = &value["data"]["records"][0];

    assert_eq!(value["data"]["view"], "sequence_debug.v0");
    assert_eq!(record["id"], "bad");
    assert_eq!(record["normalized_sequence"], "AX*");
    assert_eq!(record["token_map"][0]["status"], "standard");
    assert_eq!(record["token_map"][1]["status"], "warning");
    assert_eq!(record["token_map"][2]["status"], "error");
    assert_eq!(record["model_input"], Value::Null);
    assert!(record["error_visualization"]["markers"]
        .as_str()
        .expect("markers")
        .contains('E'));
}

use biors_core::runtime::{
    Backend, BackendCapabilities, BackendConfig, ExecutionContext, ExecutionMetadata,
    ExternalProcessBackend, ExternalProcessConfig,
};
use std::path::PathBuf;
use std::process::Command;

fn runtime_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/runtime_process.py")
}

fn python3_program() -> PathBuf {
    let output = Command::new("python3")
        .arg("-c")
        .arg("import sys; print(sys.executable)")
        .output()
        .expect("resolve python3 executable");
    assert!(
        output.status.success(),
        "python3 executable resolution should succeed"
    );
    PathBuf::from(
        String::from_utf8(output.stdout)
            .expect("python3 path should be UTF-8")
            .trim(),
    )
}

fn external_process_config(mode: &str) -> ExternalProcessConfig {
    ExternalProcessConfig {
        program: python3_program(),
        args: vec![runtime_fixture().display().to_string(), mode.to_string()],
        current_dir: None,
        environment: vec![ExecutionMetadata {
            key: "BIORS_RUNTIME_ALLOWED".to_string(),
            value: "yes".to_string(),
        }],
        inherit_environment: false,
        timeout_millis: 2_000,
        max_stdout_bytes: 16 * 1024,
        max_stderr_bytes: 1024,
    }
}

fn external_process_backend(mode: &str) -> ExternalProcessBackend {
    ExternalProcessBackend::new(
        BackendConfig {
            backend_id: format!("python-{mode}"),
            provider: "external-process".to_string(),
            version: Some("0.39-test".to_string()),
            model_artifact: None,
        },
        BackendCapabilities {
            deterministic: true,
            supports_batch: true,
            supports_streaming: false,
            supported_inputs: vec!["biors.model-input.v0".to_string()],
            supported_outputs: vec!["biors.echo.v0".to_string()],
            max_input_bytes: Some(4096),
        },
        external_process_config(mode),
    )
}

#[test]
fn external_process_backend_round_trips_context_over_stdin_and_stdout_json() {
    let backend = external_process_backend("echo");
    let context = ExecutionContext {
        trace_id: Some("trace-runtime-process-001".to_string()),
        input_format: "biors.model-input.v0".to_string(),
        requested_output_format: Some("biors.echo.v0".to_string()),
        payload: br#"{"input_ids":[1,2,3]}"#.to_vec(),
        metadata: vec![ExecutionMetadata {
            key: "sequence_id".to_string(),
            value: "tiny-protein".to_string(),
        }],
    };

    let result = backend
        .execute_checked(context)
        .expect("external process backend should execute fixture");

    assert_eq!(
        result.trace_id.as_deref(),
        Some("trace-runtime-process-001")
    );
    assert_eq!(result.output_format, "biors.echo.v0");
    assert_eq!(result.payload, br#"{"input_ids":[1,2,3]}"#);
    assert!(result
        .metadata
        .iter()
        .any(|item| { item.key == "external_process_fixture" && item.value == "echo" }));
    assert!(result
        .metadata
        .iter()
        .any(|item| { item.key == "explicit_env_visible" && item.value == "yes" }));
    assert!(result
        .metadata
        .iter()
        .any(|item| item.key == "external_process.elapsed_millis"));
    assert!(result
        .metadata
        .iter()
        .any(|item| item.key == "external_process.stdout_bytes"));
    assert!(result
        .metadata
        .iter()
        .any(|item| item.key == "external_process.stderr_bytes"));
}

#[test]
fn external_process_backend_clears_parent_environment_by_default() {
    std::env::set_var("BIORS_RUNTIME_PARENT_SECRET", "should-not-leak");
    let backend = external_process_backend("echo");
    let context = ExecutionContext {
        trace_id: Some("trace-runtime-process-env-clear".to_string()),
        input_format: "biors.model-input.v0".to_string(),
        requested_output_format: Some("biors.echo.v0".to_string()),
        payload: b"{}".to_vec(),
        metadata: Vec::new(),
    };

    let result = backend
        .execute_checked(context)
        .expect("external process backend should execute fixture");
    std::env::remove_var("BIORS_RUNTIME_PARENT_SECRET");

    assert!(result
        .metadata
        .iter()
        .any(|item| { item.key == "explicit_env_visible" && item.value == "yes" }));
    assert!(result
        .metadata
        .iter()
        .any(|item| { item.key == "parent_secret_visible" && item.value == "no" }));
}

#[test]
fn external_process_backend_rejects_non_zero_exit_without_leaking_stderr_content() {
    let backend = external_process_backend("fail");
    let context = ExecutionContext {
        trace_id: Some("trace-runtime-process-002".to_string()),
        input_format: "biors.model-input.v0".to_string(),
        requested_output_format: Some("biors.echo.v0".to_string()),
        payload: b"{}".to_vec(),
        metadata: Vec::new(),
    };

    let error = backend
        .execute_checked(context)
        .expect_err("fixture exits non-zero");

    assert_eq!(error.backend_id, "python-fail");
    assert_eq!(error.code, "runtime.process_exit_failed");
    assert!(error.message.contains("status 7"));
    assert!(error.message.contains("stderr"));
    assert!(!error.message.contains("ACDEFG"));
}

#[test]
fn external_process_backend_reports_exit_failure_when_child_exits_before_reading_stdin() {
    let mut backend = external_process_backend("early-fail");
    backend.capabilities.max_input_bytes = Some(2 * 1024 * 1024);
    let context = ExecutionContext {
        trace_id: Some("trace-runtime-process-early-fail".to_string()),
        input_format: "biors.model-input.v0".to_string(),
        requested_output_format: Some("biors.echo.v0".to_string()),
        payload: vec![b'A'; 1024 * 1024],
        metadata: Vec::new(),
    };

    let error = backend
        .execute_checked(context)
        .expect_err("fixture exits before reading stdin");

    assert_eq!(error.backend_id, "python-early-fail");
    assert_eq!(error.code, "runtime.process_exit_failed");
    assert!(error.message.contains("status 9"));
}

#[test]
fn external_process_backend_times_out_and_reports_backend_id() {
    let mut backend = external_process_backend("sleep");
    backend.process.timeout_millis = 25;
    backend.capabilities.max_input_bytes = Some(2 * 1024 * 1024);
    let context = ExecutionContext {
        trace_id: Some("trace-runtime-process-003".to_string()),
        input_format: "biors.model-input.v0".to_string(),
        requested_output_format: Some("biors.echo.v0".to_string()),
        payload: vec![b'A'; 1024 * 1024],
        metadata: Vec::new(),
    };

    let error = backend
        .execute_checked(context)
        .expect_err("fixture should exceed timeout");

    assert_eq!(error.backend_id, "python-sleep");
    assert_eq!(error.code, "runtime.process_timeout");
    assert!(error.message.contains("25 ms"));
}

#[test]
fn external_process_backend_rejects_invalid_stdout_json() {
    let backend = external_process_backend("invalid-output");
    let context = ExecutionContext {
        trace_id: None,
        input_format: "biors.model-input.v0".to_string(),
        requested_output_format: Some("biors.echo.v0".to_string()),
        payload: b"{}".to_vec(),
        metadata: Vec::new(),
    };

    let error = backend
        .execute_checked(context)
        .expect_err("fixture writes invalid JSON");

    assert_eq!(error.backend_id, "python-invalid-output");
    assert_eq!(error.code, "runtime.process_invalid_output");
}

#[test]
fn external_process_backend_rejects_wrong_requested_output_format() {
    let mut backend = external_process_backend("wrong-output");
    backend
        .capabilities
        .supported_outputs
        .push("biors.alternate.v0".to_string());
    let context = ExecutionContext {
        trace_id: Some("trace-runtime-process-output-format".to_string()),
        input_format: "biors.model-input.v0".to_string(),
        requested_output_format: Some("biors.echo.v0".to_string()),
        payload: b"{}".to_vec(),
        metadata: Vec::new(),
    };

    let error = backend
        .execute_checked(context)
        .expect_err("fixture returns a supported but unrequested output format");

    assert_eq!(error.backend_id, "python-wrong-output");
    assert_eq!(error.code, "runtime.output_format_mismatch");
    assert!(error.message.contains("biors.echo.v0"));
    assert!(error.message.contains("biors.alternate.v0"));
}

#[test]
fn external_process_backend_bounds_stdout_before_parsing_result() {
    let mut backend = external_process_backend("big-stdout");
    backend.process.max_stdout_bytes = 128;
    let context = ExecutionContext {
        trace_id: None,
        input_format: "biors.model-input.v0".to_string(),
        requested_output_format: Some("biors.echo.v0".to_string()),
        payload: b"{}".to_vec(),
        metadata: Vec::new(),
    };

    let error = backend
        .execute_checked(context)
        .expect_err("fixture writes more stdout than configured limit");

    assert_eq!(error.backend_id, "python-big-stdout");
    assert_eq!(error.code, "runtime.process_stdout_too_large");
    assert!(error.message.contains("128 bytes"));
}

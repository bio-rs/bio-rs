use super::contracts::BackendExecutionError;
use std::fmt;

impl BackendExecutionError {
    /// Construct an unsupported-input error for capability checks.
    pub fn unsupported_input(backend_id: &str, input_format: &str) -> Self {
        Self {
            backend_id: backend_id.to_string(),
            code: "runtime.unsupported_input".to_string(),
            message: format!(
                "backend '{backend_id}' does not support input format '{input_format}'"
            ),
        }
    }

    /// Construct an unsupported-output error for capability checks.
    pub fn unsupported_output(backend_id: &str, output_format: &str) -> Self {
        Self {
            backend_id: backend_id.to_string(),
            code: "runtime.unsupported_output".to_string(),
            message: format!(
                "backend '{backend_id}' does not support output format '{output_format}'"
            ),
        }
    }

    /// Construct a payload-size error for capability checks.
    pub fn payload_too_large(
        backend_id: &str,
        payload_bytes: usize,
        max_input_bytes: usize,
    ) -> Self {
        Self {
            backend_id: backend_id.to_string(),
            code: "runtime.payload_too_large".to_string(),
            message: format!(
                "backend '{backend_id}' rejected {payload_bytes} bytes; limit is {max_input_bytes} bytes"
            ),
        }
    }

    /// Construct a generic backend execution failure.
    pub fn execution_failed(backend_id: &str, message: impl Into<String>) -> Self {
        Self {
            backend_id: backend_id.to_string(),
            code: "runtime.execution_failed".to_string(),
            message: message.into(),
        }
    }

    pub fn process_spawn_failed(backend_id: &str, message: impl Into<String>) -> Self {
        Self::process_error(backend_id, "runtime.process_spawn_failed", message)
    }

    pub fn process_io_failed(backend_id: &str, message: impl Into<String>) -> Self {
        Self::process_error(backend_id, "runtime.process_io_failed", message)
    }

    pub fn process_timeout(backend_id: &str, timeout_millis: u64) -> Self {
        Self::process_error(
            backend_id,
            "runtime.process_timeout",
            format!("external process exceeded timeout of {timeout_millis} ms"),
        )
    }

    pub fn process_exit_failed(backend_id: &str, status: &str, stderr_bytes: usize) -> Self {
        Self::process_error(
            backend_id,
            "runtime.process_exit_failed",
            format!("external process exited with {status}; stderr {stderr_bytes} bytes captured"),
        )
    }

    pub fn process_stdout_too_large(
        backend_id: &str,
        limit_bytes: usize,
        total_bytes: usize,
    ) -> Self {
        Self::process_error(
            backend_id,
            "runtime.process_stdout_too_large",
            format!(
                "external process stdout exceeded {limit_bytes} bytes; drained {total_bytes} bytes"
            ),
        )
    }

    pub fn process_stderr_too_large(
        backend_id: &str,
        limit_bytes: usize,
        total_bytes: usize,
    ) -> Self {
        Self::process_error(
            backend_id,
            "runtime.process_stderr_too_large",
            format!(
                "external process stderr exceeded {limit_bytes} bytes; drained {total_bytes} bytes"
            ),
        )
    }

    pub fn process_invalid_output(
        backend_id: &str,
        output_bytes: usize,
        message: impl Into<String>,
    ) -> Self {
        Self::process_error(
            backend_id,
            "runtime.process_invalid_output",
            format!(
                "external process stdout ({output_bytes} bytes) was not a valid ExecutionResult JSON: {}",
                message.into()
            ),
        )
    }

    fn process_error(backend_id: &str, code: &str, message: impl Into<String>) -> Self {
        Self {
            backend_id: backend_id.to_string(),
            code: code.to_string(),
            message: message.into(),
        }
    }
}

impl fmt::Display for BackendExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for BackendExecutionError {}

use serde::{Deserialize, Serialize};

/// Minimal execution backend interface.
pub trait Backend {
    /// Static backend configuration.
    fn config(&self) -> &BackendConfig;

    /// Declared backend capability contract.
    fn capabilities(&self) -> &BackendCapabilities;

    /// Execute one request against the backend.
    fn execute(&self, context: ExecutionContext) -> Result<ExecutionResult, BackendExecutionError>;

    /// Validate a context against declared capabilities before executing it.
    fn execute_checked(
        &self,
        context: ExecutionContext,
    ) -> Result<ExecutionResult, BackendExecutionError> {
        self.capabilities()
            .ensure_context(&self.config().backend_id, &context)?;
        self.execute(context)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Static backend identity and model binding.
pub struct BackendConfig {
    /// Stable backend identifier used in logs, errors, and compatibility checks.
    pub backend_id: String,
    /// Human-readable provider or implementation family.
    pub provider: String,
    /// Optional backend implementation version.
    pub version: Option<String>,
    /// Optional package-relative or host-local model artifact reference.
    pub model_artifact: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Backend input/output and behavior capabilities.
pub struct BackendCapabilities {
    /// True when the backend is expected to return reproducible output for the
    /// same context and artifact state.
    pub deterministic: bool,
    /// True when one request can contain multiple independent examples.
    pub supports_batch: bool,
    /// True when the backend can stream partial output.
    pub supports_streaming: bool,
    /// Stable input formats accepted by the backend.
    pub supported_inputs: Vec<String>,
    /// Stable output formats the backend can produce.
    pub supported_outputs: Vec<String>,
    /// Optional request payload size limit.
    pub max_input_bytes: Option<usize>,
}

impl BackendCapabilities {
    /// Return true when the backend declares support for `format`.
    pub fn supports_input(&self, format: &str) -> bool {
        self.supported_inputs
            .iter()
            .any(|supported| supported == format)
    }

    /// Return true when the backend declares support for `format`.
    pub fn supports_output(&self, format: &str) -> bool {
        self.supported_outputs
            .iter()
            .any(|supported| supported == format)
    }

    /// Report all context compatibility issues without executing a backend.
    pub fn compatibility_report(&self, context: &ExecutionContext) -> BackendCompatibilityReport {
        let mut issues = Vec::new();

        if !self.supports_input(&context.input_format) {
            issues.push(RuntimeCompatibilityIssue {
                code: RuntimeCompatibilityIssueCode::UnsupportedInput,
                message: format!(
                    "input format '{}' is not declared in supported_inputs",
                    context.input_format
                ),
            });
        }

        if let Some(requested_output) = &context.requested_output_format {
            if !self.supports_output(requested_output) {
                issues.push(RuntimeCompatibilityIssue {
                    code: RuntimeCompatibilityIssueCode::UnsupportedOutput,
                    message: format!(
                        "requested output format '{requested_output}' is not declared in supported_outputs"
                    ),
                });
            }
        }

        if let Some(max_input_bytes) = self.max_input_bytes {
            if context.payload.len() > max_input_bytes {
                issues.push(RuntimeCompatibilityIssue {
                    code: RuntimeCompatibilityIssueCode::PayloadTooLarge,
                    message: format!(
                        "payload is {} bytes but backend limit is {max_input_bytes} bytes",
                        context.payload.len()
                    ),
                });
            }
        }

        BackendCompatibilityReport {
            compatible: issues.is_empty(),
            issues,
        }
    }

    /// Return the first execution error that would block this context.
    pub fn ensure_context(
        &self,
        backend_id: &str,
        context: &ExecutionContext,
    ) -> Result<(), BackendExecutionError> {
        let report = self.compatibility_report(context);
        if let Some(issue) = report.issues.into_iter().next() {
            return Err(match issue.code {
                RuntimeCompatibilityIssueCode::UnsupportedInput => {
                    BackendExecutionError::unsupported_input(backend_id, &context.input_format)
                }
                RuntimeCompatibilityIssueCode::UnsupportedOutput => {
                    BackendExecutionError::unsupported_output(
                        backend_id,
                        context
                            .requested_output_format
                            .as_deref()
                            .unwrap_or("<unspecified>"),
                    )
                }
                RuntimeCompatibilityIssueCode::PayloadTooLarge => {
                    BackendExecutionError::payload_too_large(
                        backend_id,
                        context.payload.len(),
                        self.max_input_bytes.unwrap_or(0),
                    )
                }
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Compatibility report for checking a context before backend execution.
pub struct BackendCompatibilityReport {
    pub compatible: bool,
    pub issues: Vec<RuntimeCompatibilityIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// One compatibility issue found before backend execution.
pub struct RuntimeCompatibilityIssue {
    pub code: RuntimeCompatibilityIssueCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Stable compatibility issue codes for runtime preflight checks.
pub enum RuntimeCompatibilityIssueCode {
    UnsupportedInput,
    UnsupportedOutput,
    PayloadTooLarge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// One execution request passed to a backend.
pub struct ExecutionContext {
    /// Optional caller-supplied trace identifier.
    pub trace_id: Option<String>,
    /// Stable input payload format.
    pub input_format: String,
    /// Optional requested output payload format.
    pub requested_output_format: Option<String>,
    /// Opaque payload owned by the backend contract.
    pub payload: Vec<u8>,
    /// Small structured metadata fields safe to echo into logs or reports.
    pub metadata: Vec<ExecutionMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// String metadata carried with an execution request or result.
pub struct ExecutionMetadata {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// One execution response returned by a backend.
pub struct ExecutionResult {
    /// Trace identifier propagated from the execution context when available.
    pub trace_id: Option<String>,
    /// Stable output payload format.
    pub output_format: String,
    /// Opaque result payload owned by the backend contract.
    pub payload: Vec<u8>,
    /// Small structured metadata fields safe to echo into logs or reports.
    pub metadata: Vec<ExecutionMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Stable backend execution error.
pub struct BackendExecutionError {
    pub backend_id: String,
    pub code: String,
    pub message: String,
}

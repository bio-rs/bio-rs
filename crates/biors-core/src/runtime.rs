//! Runtime backend abstraction contracts.
//!
//! This module defines stable core types a backend implementation must use and
//! provides a guarded external-process backend for local runtime integration.

mod contracts;
mod errors;
mod process;
mod process_io;

pub use contracts::{
    Backend, BackendCapabilities, BackendCompatibilityReport, BackendConfig, BackendExecutionError,
    ExecutionContext, ExecutionMetadata, ExecutionResult, RuntimeCompatibilityIssue,
    RuntimeCompatibilityIssueCode,
};
pub use process::{
    ExternalProcessBackend, ExternalProcessConfig, DEFAULT_PROCESS_TIMEOUT_MILLIS,
    DEFAULT_STDERR_LIMIT_BYTES, DEFAULT_STDOUT_LIMIT_BYTES,
};

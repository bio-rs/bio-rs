use super::process_io::{
    format_exit_status, join_capture, read_limited, wait_for_child, ChildWaitResult,
};
use super::{
    Backend, BackendCapabilities, BackendConfig, BackendExecutionError, ExecutionContext,
    ExecutionMetadata, ExecutionResult,
};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub const DEFAULT_PROCESS_TIMEOUT_MILLIS: u64 = 30_000;
pub const DEFAULT_STDOUT_LIMIT_BYTES: usize = 16 * 1024 * 1024;
pub const DEFAULT_STDERR_LIMIT_BYTES: usize = 1024 * 1024;

/// External process invocation policy for a runtime backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalProcessConfig {
    /// Program path executed directly without a shell.
    pub program: PathBuf,
    /// Program arguments passed without shell interpolation.
    pub args: Vec<String>,
    /// Optional child working directory.
    pub current_dir: Option<PathBuf>,
    /// Explicit child environment variables.
    pub environment: Vec<ExecutionMetadata>,
    /// Whether to inherit the parent process environment.
    pub inherit_environment: bool,
    /// Wall-clock timeout for one execution.
    pub timeout_millis: u64,
    /// Maximum stdout bytes retained before JSON parsing.
    pub max_stdout_bytes: usize,
    /// Maximum stderr bytes retained for diagnostics accounting.
    pub max_stderr_bytes: usize,
}

impl ExternalProcessConfig {
    /// Build a guarded external process config with conservative defaults.
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            current_dir: None,
            environment: Vec::new(),
            inherit_environment: false,
            timeout_millis: DEFAULT_PROCESS_TIMEOUT_MILLIS,
            max_stdout_bytes: DEFAULT_STDOUT_LIMIT_BYTES,
            max_stderr_bytes: DEFAULT_STDERR_LIMIT_BYTES,
        }
    }
}

/// Backend implementation that delegates execution to a local child process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalProcessBackend {
    pub config: BackendConfig,
    pub capabilities: BackendCapabilities,
    pub process: ExternalProcessConfig,
}

impl ExternalProcessBackend {
    pub fn new(
        config: BackendConfig,
        capabilities: BackendCapabilities,
        process: ExternalProcessConfig,
    ) -> Self {
        Self {
            config,
            capabilities,
            process,
        }
    }
}

impl Backend for ExternalProcessBackend {
    fn config(&self) -> &BackendConfig {
        &self.config
    }

    fn capabilities(&self) -> &BackendCapabilities {
        &self.capabilities
    }

    fn execute(&self, context: ExecutionContext) -> Result<ExecutionResult, BackendExecutionError> {
        let started_at = Instant::now();
        let stdin = serde_json::to_vec(&context).map_err(|error| {
            BackendExecutionError::process_io_failed(
                &self.config.backend_id,
                format!("failed to serialize execution context: {error}"),
            )
        })?;

        let mut command = Command::new(&self.process.program);
        command
            .args(&self.process.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if !self.process.inherit_environment {
            command.env_clear();
        }
        for item in &self.process.environment {
            command.env(&item.key, &item.value);
        }
        if let Some(current_dir) = &self.process.current_dir {
            command.current_dir(current_dir);
        }

        let mut child = command.spawn().map_err(|error| {
            BackendExecutionError::process_spawn_failed(
                &self.config.backend_id,
                format!(
                    "failed to spawn external process '{}': {error}",
                    self.process.program.display()
                ),
            )
        })?;

        let child_stdin = child.stdin.take().ok_or_else(|| {
            BackendExecutionError::process_io_failed(&self.config.backend_id, "child stdin missing")
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            BackendExecutionError::process_io_failed(
                &self.config.backend_id,
                "child stdout missing",
            )
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            BackendExecutionError::process_io_failed(
                &self.config.backend_id,
                "child stderr missing",
            )
        })?;
        let stdout_limit = self.process.max_stdout_bytes;
        let stderr_limit = self.process.max_stderr_bytes;

        let stdout_reader = thread::spawn(move || read_limited(stdout, stdout_limit));
        let stderr_reader = thread::spawn(move || read_limited(stderr, stderr_limit));
        let stdin_writer = thread::spawn(move || write_stdin(child_stdin, stdin));

        let status = wait_for_child(
            &mut child,
            Duration::from_millis(self.process.timeout_millis),
        )
        .map_err(|error| {
            BackendExecutionError::process_io_failed(
                &self.config.backend_id,
                format!("failed while waiting for external process: {error}"),
            )
        })?;

        let stdout_capture = join_capture(stdout_reader, &self.config.backend_id, "stdout")?;
        let stderr_capture = join_capture(stderr_reader, &self.config.backend_id, "stderr")?;
        let stdin_write = join_stdin_writer(stdin_writer, &self.config.backend_id);

        let status = match status {
            ChildWaitResult::Exited(status) => status,
            ChildWaitResult::TimedOut => {
                let _ = stdin_write;
                return Err(BackendExecutionError::process_timeout(
                    &self.config.backend_id,
                    self.process.timeout_millis,
                ));
            }
        };

        if stderr_capture.exceeded {
            return Err(BackendExecutionError::process_stderr_too_large(
                &self.config.backend_id,
                self.process.max_stderr_bytes,
                stderr_capture.total_bytes,
            ));
        }
        if stdout_capture.exceeded {
            return Err(BackendExecutionError::process_stdout_too_large(
                &self.config.backend_id,
                self.process.max_stdout_bytes,
                stdout_capture.total_bytes,
            ));
        }
        if !status.success() {
            return Err(BackendExecutionError::process_exit_failed(
                &self.config.backend_id,
                &format_exit_status(status),
                stderr_capture.total_bytes,
            ));
        }

        stdin_write?;

        let mut result: ExecutionResult =
            serde_json::from_slice(&stdout_capture.bytes).map_err(|error| {
                BackendExecutionError::process_invalid_output(
                    &self.config.backend_id,
                    stdout_capture.total_bytes,
                    error.to_string(),
                )
            })?;

        if !self.capabilities.supports_output(&result.output_format) {
            return Err(BackendExecutionError::unsupported_output(
                &self.config.backend_id,
                &result.output_format,
            ));
        }
        if result.trace_id.is_none() {
            result.trace_id = context.trace_id;
        }
        append_process_metadata(
            &mut result,
            started_at.elapsed(),
            stdout_capture.total_bytes,
            stderr_capture.total_bytes,
            &status,
        );

        Ok(result)
    }
}

fn write_stdin(mut child_stdin: std::process::ChildStdin, stdin: Vec<u8>) -> io::Result<()> {
    child_stdin.write_all(&stdin)
}

fn join_stdin_writer(
    handle: thread::JoinHandle<io::Result<()>>,
    backend_id: &str,
) -> Result<(), BackendExecutionError> {
    handle
        .join()
        .map_err(|_| {
            BackendExecutionError::process_io_failed(
                backend_id,
                "external process stdin writer panicked",
            )
        })?
        .map_err(|error| {
            BackendExecutionError::process_io_failed(
                backend_id,
                format!("failed to write execution context to child stdin: {error}"),
            )
        })
}

fn append_process_metadata(
    result: &mut ExecutionResult,
    elapsed: Duration,
    stdout_bytes: usize,
    stderr_bytes: usize,
    status: &ExitStatus,
) {
    result.metadata.push(ExecutionMetadata {
        key: "external_process.elapsed_millis".to_string(),
        value: elapsed.as_millis().to_string(),
    });
    result.metadata.push(ExecutionMetadata {
        key: "external_process.stdout_bytes".to_string(),
        value: stdout_bytes.to_string(),
    });
    result.metadata.push(ExecutionMetadata {
        key: "external_process.stderr_bytes".to_string(),
        value: stderr_bytes.to_string(),
    });
    result.metadata.push(ExecutionMetadata {
        key: "external_process.exit_status".to_string(),
        value: format_exit_status(*status),
    });
}

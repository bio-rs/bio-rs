use super::BackendExecutionError;
use std::io::{self, Read};
use std::process::ExitStatus;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(super) struct OutputCapture {
    pub bytes: Vec<u8>,
    pub total_bytes: usize,
    pub exceeded: bool,
}

pub(super) fn read_limited(mut reader: impl Read, limit: usize) -> io::Result<OutputCapture> {
    let mut bytes = Vec::with_capacity(limit.min(8192));
    let mut buffer = [0_u8; 8192];
    let mut total_bytes = 0;
    let mut exceeded = false;

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }

        total_bytes += read;
        if bytes.len() < limit {
            let remaining = limit - bytes.len();
            let keep = remaining.min(read);
            bytes.extend_from_slice(&buffer[..keep]);
        }
        if total_bytes > limit {
            exceeded = true;
        }
    }

    Ok(OutputCapture {
        bytes,
        total_bytes,
        exceeded,
    })
}

pub(super) enum ChildWaitResult {
    Exited(ExitStatus),
    TimedOut,
}

pub(super) fn wait_for_child(
    child: &mut std::process::Child,
    timeout: Duration,
) -> io::Result<ChildWaitResult> {
    let deadline = Instant::now() + timeout;

    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(ChildWaitResult::Exited(status));
        }

        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(ChildWaitResult::TimedOut);
        }

        thread::sleep(Duration::from_millis(5));
    }
}

pub(super) fn join_capture(
    handle: thread::JoinHandle<io::Result<OutputCapture>>,
    backend_id: &str,
    stream_name: &str,
) -> Result<OutputCapture, BackendExecutionError> {
    handle
        .join()
        .map_err(|_| {
            BackendExecutionError::process_io_failed(
                backend_id,
                format!("external process {stream_name} reader panicked"),
            )
        })?
        .map_err(|error| {
            BackendExecutionError::process_io_failed(
                backend_id,
                format!("failed to read external process {stream_name}: {error}"),
            )
        })
}

pub(super) fn format_exit_status(status: ExitStatus) -> String {
    if let Some(code) = status.code() {
        format!("status {code}")
    } else {
        status.to_string()
    }
}

use crate::errors::{CliError, ErrorLocationValue};
use serde::Serialize;
use std::io::{self, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize)]
struct CliSuccess<T: Serialize> {
    ok: bool,
    biors_version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_hash: Option<String>,
    data: T,
}

#[derive(Debug, Serialize)]
struct CliFailure {
    ok: bool,
    error: CliErrorBody,
}

#[derive(Debug, Serialize)]
struct CliErrorBody {
    code: &'static str,
    message: String,
    location: Option<ErrorLocationValue>,
}

pub(crate) fn print_success<T: Serialize>(
    input_hash: Option<String>,
    data: T,
) -> Result<(), CliError> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write_success_to(&mut handle, input_hash, data)?;
    Ok(())
}

fn write_success_to<W: Write, T: Serialize>(
    writer: &mut W,
    input_hash: Option<String>,
    data: T,
) -> Result<(), CliError> {
    let payload = CliSuccess {
        ok: true,
        biors_version: VERSION,
        input_hash,
        data,
    };
    serde_json::to_writer_pretty(&mut *writer, &payload).map_err(CliError::Serialization)?;
    writeln!(writer).map_err(CliError::Write)?;
    Ok(())
}

pub(crate) fn print_json_error(error: CliError) {
    let payload = CliFailure {
        ok: false,
        error: CliErrorBody {
            code: error.code(),
            message: error.to_string(),
            location: error.location(),
        },
    };
    println!("{}", to_json(&payload).expect("serialize JSON error"));
}

fn to_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn write_success_to_serializes_envelope_to_writer() {
        let mut output = Vec::new();

        write_success_to(&mut output, Some("fnv1a64:test".to_string()), vec![1, 2])
            .expect("write success envelope");

        assert!(output.ends_with(b"\n"));
        let value: Value = serde_json::from_slice(&output).expect("valid JSON");
        assert_eq!(value["ok"], true);
        assert_eq!(value["biors_version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(value["input_hash"], "fnv1a64:test");
        assert_eq!(value["data"], serde_json::json!([1, 2]));
    }
}

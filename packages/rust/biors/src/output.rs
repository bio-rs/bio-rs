use crate::errors::{CliError, ErrorLocationValue};
use serde::Serialize;

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
    let payload = CliSuccess {
        ok: true,
        biors_version: VERSION,
        input_hash,
        data,
    };
    println!("{}", to_json(&payload)?);
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

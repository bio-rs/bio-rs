use std::fmt;

pub const CANDLE_LOAD_FAILED: &str = "candle.load_failed";
pub const CANDLE_MISSING_TENSOR: &str = "candle.missing_tensor";
pub const CANDLE_INVALID_SHAPE: &str = "candle.invalid_shape";
pub const CANDLE_INVALID_DTYPE: &str = "candle.invalid_dtype";
pub const CANDLE_TOKEN_ID_OUT_OF_RANGE: &str = "candle.token_id_out_of_range";
pub const CANDLE_TENSOR_FAILED: &str = "candle.tensor_failed";
pub const CANDLE_INFERENCE_FAILED: &str = "candle.inference_failed";
pub const CANDLE_OUTPUT_FAILED: &str = "candle.output_failed";

pub const CANDLE_ERROR_CODES: &[&str] = &[
    CANDLE_LOAD_FAILED,
    CANDLE_MISSING_TENSOR,
    CANDLE_INVALID_SHAPE,
    CANDLE_INVALID_DTYPE,
    CANDLE_TOKEN_ID_OUT_OF_RANGE,
    CANDLE_TENSOR_FAILED,
    CANDLE_INFERENCE_FAILED,
    CANDLE_OUTPUT_FAILED,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandleBackendError {
    pub code: String,
    pub message: String,
}

impl CandleBackendError {
    pub(crate) fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

impl fmt::Display for CandleBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for CandleBackendError {}

pub(crate) fn candle_error(code: &str, error: candle_core::Error) -> CandleBackendError {
    CandleBackendError::new(code, error.to_string())
}

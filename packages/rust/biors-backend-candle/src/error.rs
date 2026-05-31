use std::fmt;

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

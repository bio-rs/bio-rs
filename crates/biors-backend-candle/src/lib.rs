//! Optional Candle backend adapter for bio-rs runtime contracts.

mod backend;
mod config;
mod error;
mod output;
mod tensor;

pub use backend::CandleBackend;
pub use config::{CandleBackendConfig, CandleDevice};
pub use error::{CandleBackendError, CANDLE_ERROR_CODES};
pub use output::{CandleInferenceOutput, CandleInferenceRecord};

pub const CANDLE_MODEL_INPUT_FORMAT: &str = "biors.model-input.v0+json";
pub const CANDLE_OUTPUT_FORMAT: &str = "biors.candle.linear-probe.v0+json";

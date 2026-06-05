use candle_core::Device;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandleDevice {
    Cpu,
}

impl CandleDevice {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
        }
    }

    pub(crate) fn candle_device(self) -> Device {
        match self {
            Self::Cpu => Device::Cpu,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandleBackendConfig {
    pub backend_id: String,
    pub weights_path: PathBuf,
    pub device: CandleDevice,
    pub embedding_tensor: String,
    pub projection_weight_tensor: String,
    pub projection_bias_tensor: Option<String>,
    pub max_input_bytes: Option<usize>,
}

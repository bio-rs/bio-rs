use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandleInferenceOutput {
    pub records: Vec<CandleInferenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandleInferenceRecord {
    pub id: String,
    pub values: Vec<f32>,
    pub truncated: bool,
}

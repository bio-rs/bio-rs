//! Optional Candle backend adapter for bio-rs runtime contracts.

use biors_core::model_input::{ModelInput, ModelInputRecord};
use biors_core::runtime::{
    Backend, BackendCapabilities, BackendConfig, BackendExecutionError, ExecutionContext,
    ExecutionMetadata, ExecutionResult,
};
use candle_core::{safetensors, DType, Device, Tensor};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::time::Instant;

pub const CANDLE_MODEL_INPUT_FORMAT: &str = "biors.model-input.v0+json";
pub const CANDLE_OUTPUT_FORMAT: &str = "biors.candle.linear-probe.v0+json";

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

    fn candle_device(self) -> Device {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandleBackendError {
    pub code: String,
    pub message: String,
}

impl CandleBackendError {
    fn new(code: &str, message: impl Into<String>) -> Self {
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

pub struct CandleBackend {
    runtime_config: BackendConfig,
    capabilities: BackendCapabilities,
    config: CandleBackendConfig,
    device: Device,
    embedding: Tensor,
    projection_weight: Tensor,
    projection_bias: Option<Tensor>,
}

impl CandleBackend {
    pub fn from_safetensors(config: CandleBackendConfig) -> Result<Self, CandleBackendError> {
        let device = config.device.candle_device();
        let mut tensors = safetensors::load(&config.weights_path, &device).map_err(|error| {
            CandleBackendError::new(
                "candle.load_failed",
                format!(
                    "failed to load Candle safetensors '{}': {error}",
                    config.weights_path.display()
                ),
            )
        })?;

        let embedding = take_tensor(&mut tensors, &config.embedding_tensor)?;
        let projection_weight = take_tensor(&mut tensors, &config.projection_weight_tensor)?;
        let projection_bias = match &config.projection_bias_tensor {
            Some(name) => Some(take_tensor(&mut tensors, name)?),
            None => None,
        };

        validate_model_tensors(&embedding, &projection_weight, projection_bias.as_ref())?;

        Ok(Self {
            runtime_config: BackendConfig {
                backend_id: config.backend_id.clone(),
                provider: "candle".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                model_artifact: Some(config.weights_path.display().to_string()),
            },
            capabilities: BackendCapabilities {
                deterministic: true,
                supports_batch: true,
                supports_streaming: false,
                supported_inputs: vec![CANDLE_MODEL_INPUT_FORMAT.to_string()],
                supported_outputs: vec![CANDLE_OUTPUT_FORMAT.to_string()],
                max_input_bytes: config.max_input_bytes,
            },
            device,
            embedding,
            projection_weight,
            projection_bias,
            config,
        })
    }

    pub fn candle_config(&self) -> &CandleBackendConfig {
        &self.config
    }

    fn execute_model_input(
        &self,
        input: &ModelInput,
    ) -> Result<CandleInferenceOutput, CandleBackendError> {
        let records = input
            .records
            .iter()
            .map(|record| self.score_record(record))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CandleInferenceOutput { records })
    }

    fn score_record(
        &self,
        record: &ModelInputRecord,
    ) -> Result<CandleInferenceRecord, CandleBackendError> {
        if record.input_ids.len() != record.attention_mask.len() {
            return Err(CandleBackendError::new(
                "candle.invalid_model_input",
                format!(
                    "record '{}' has {} input ids but {} attention-mask values",
                    record.id,
                    record.input_ids.len(),
                    record.attention_mask.len()
                ),
            ));
        }

        let vocab_size = self.embedding.dims()[0];
        let ids = record
            .input_ids
            .iter()
            .zip(&record.attention_mask)
            .filter_map(|(token_id, mask)| {
                if *mask == 0 {
                    None
                } else {
                    Some(*token_id as u32)
                }
            })
            .collect::<Vec<_>>();

        if ids.is_empty() {
            return Err(CandleBackendError::new(
                "candle.empty_attention_mask",
                format!("record '{}' has no unmasked tokens", record.id),
            ));
        }

        if let Some(token_id) = ids
            .iter()
            .find(|token_id| **token_id as usize >= vocab_size)
        {
            return Err(CandleBackendError::new(
                "candle.token_id_out_of_range",
                format!(
                    "record '{}' token id {} exceeds embedding vocabulary size {vocab_size}",
                    record.id, token_id
                ),
            ));
        }

        let ids = Tensor::new(ids.as_slice(), &self.device)
            .map_err(|error| candle_error("candle.tensor_failed", error))?;
        let pooled = self
            .embedding
            .embedding(&ids)
            .and_then(|embedded| embedded.mean(0))
            .map_err(|error| candle_error("candle.inference_failed", error))?;
        let mut scores = pooled
            .unsqueeze(0)
            .and_then(|pooled| pooled.matmul(&self.projection_weight))
            .and_then(|projected| projected.squeeze(0))
            .map_err(|error| candle_error("candle.inference_failed", error))?;

        if let Some(bias) = &self.projection_bias {
            scores = scores
                .broadcast_add(bias)
                .map_err(|error| candle_error("candle.inference_failed", error))?;
        }

        Ok(CandleInferenceRecord {
            id: record.id.clone(),
            values: scores
                .to_dtype(DType::F32)
                .and_then(|scores| scores.to_vec1::<f32>())
                .map_err(|error| candle_error("candle.output_failed", error))?,
            truncated: record.truncated,
        })
    }
}

impl Backend for CandleBackend {
    fn config(&self) -> &BackendConfig {
        &self.runtime_config
    }

    fn capabilities(&self) -> &BackendCapabilities {
        &self.capabilities
    }

    fn execute(&self, context: ExecutionContext) -> Result<ExecutionResult, BackendExecutionError> {
        let started_at = Instant::now();
        let input: ModelInput = serde_json::from_slice(&context.payload).map_err(|error| {
            BackendExecutionError::execution_failed(
                &self.runtime_config.backend_id,
                format!("invalid ModelInput JSON payload: {error}"),
            )
        })?;
        let output = self.execute_model_input(&input).map_err(|error| {
            BackendExecutionError::execution_failed(
                &self.runtime_config.backend_id,
                error.to_string(),
            )
        })?;
        let payload = serde_json::to_vec(&output).map_err(|error| {
            BackendExecutionError::execution_failed(
                &self.runtime_config.backend_id,
                format!("failed to serialize Candle output: {error}"),
            )
        })?;

        Ok(ExecutionResult {
            trace_id: context.trace_id,
            output_format: CANDLE_OUTPUT_FORMAT.to_string(),
            payload,
            metadata: vec![
                ExecutionMetadata {
                    key: "candle.device".to_string(),
                    value: self.config.device.as_str().to_string(),
                },
                ExecutionMetadata {
                    key: "candle.elapsed_millis".to_string(),
                    value: started_at.elapsed().as_millis().to_string(),
                },
                ExecutionMetadata {
                    key: "candle.output_records".to_string(),
                    value: input.records.len().to_string(),
                },
            ],
        })
    }
}

fn take_tensor(
    tensors: &mut std::collections::HashMap<String, Tensor>,
    name: &str,
) -> Result<Tensor, CandleBackendError> {
    tensors.remove(name).ok_or_else(|| {
        CandleBackendError::new(
            "candle.missing_tensor",
            format!("safetensors file does not contain tensor '{name}'"),
        )
    })
}

fn validate_model_tensors(
    embedding: &Tensor,
    projection_weight: &Tensor,
    projection_bias: Option<&Tensor>,
) -> Result<(), CandleBackendError> {
    ensure_float_tensor("embedding", embedding)?;
    ensure_float_tensor("projection weight", projection_weight)?;

    let embedding_dims = embedding.dims();
    if embedding_dims.len() != 2 {
        return Err(CandleBackendError::new(
            "candle.invalid_shape",
            format!("embedding tensor must be rank 2, got {embedding_dims:?}"),
        ));
    }

    let projection_dims = projection_weight.dims();
    if projection_dims.len() != 2 {
        return Err(CandleBackendError::new(
            "candle.invalid_shape",
            format!("projection weight tensor must be rank 2, got {projection_dims:?}"),
        ));
    }
    if projection_dims[0] != embedding_dims[1] {
        return Err(CandleBackendError::new(
            "candle.invalid_shape",
            format!(
                "projection input dim {} does not match embedding hidden dim {}",
                projection_dims[0], embedding_dims[1]
            ),
        ));
    }

    if let Some(bias) = projection_bias {
        ensure_float_tensor("projection bias", bias)?;
        let bias_dims = bias.dims();
        if bias_dims != [projection_dims[1]] {
            return Err(CandleBackendError::new(
                "candle.invalid_shape",
                format!(
                    "projection bias tensor must have shape [{}], got {bias_dims:?}",
                    projection_dims[1]
                ),
            ));
        }
    }

    Ok(())
}

fn ensure_float_tensor(name: &str, tensor: &Tensor) -> Result<(), CandleBackendError> {
    if !matches!(
        tensor.dtype(),
        DType::F16 | DType::BF16 | DType::F32 | DType::F64
    ) {
        return Err(CandleBackendError::new(
            "candle.invalid_dtype",
            format!(
                "{name} tensor must be floating point, got {:?}",
                tensor.dtype()
            ),
        ));
    }
    Ok(())
}

fn candle_error(code: &str, error: candle_core::Error) -> CandleBackendError {
    CandleBackendError::new(code, error.to_string())
}

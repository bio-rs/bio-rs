use crate::config::CandleBackendConfig;
use crate::error::{candle_error, CandleBackendError};
use crate::output::{CandleInferenceOutput, CandleInferenceRecord};
use crate::tensor::{take_tensor, validate_model_tensors};
use crate::{CANDLE_MODEL_INPUT_FORMAT, CANDLE_OUTPUT_FORMAT};
use biors_core::model_input::{validate_model_input_payload, ModelInput, ModelInputRecord};
use biors_core::runtime::{
    Backend, BackendCapabilities, BackendConfig, BackendExecutionError, ExecutionContext,
    ExecutionMetadata, ExecutionResult,
};
use candle_core::{safetensors, DType, Device, Tensor};
use std::time::Instant;

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
        validate_model_input_payload(input).map_err(|error| {
            CandleBackendError::new(error.code(), format!("invalid ModelInput payload: {error}"))
        })?;

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

use crate::error::CandleBackendError;
use candle_core::{DType, Tensor};
use std::collections::HashMap;

pub(crate) fn take_tensor(
    tensors: &mut HashMap<String, Tensor>,
    name: &str,
) -> Result<Tensor, CandleBackendError> {
    tensors.remove(name).ok_or_else(|| {
        CandleBackendError::new(
            "candle.missing_tensor",
            format!("safetensors file does not contain tensor '{name}'"),
        )
    })
}

pub(crate) fn validate_model_tensors(
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

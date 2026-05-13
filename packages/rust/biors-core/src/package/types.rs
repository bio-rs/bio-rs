use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaVersion {
    #[serde(rename = "biors.package.v0")]
    BiorsPackageV0,
    #[serde(rename = "biors.package.v1")]
    BiorsPackageV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFormat {
    #[serde(rename = "onnx")]
    Onnx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeBackend {
    #[serde(rename = "onnx-webgpu")]
    OnnxWebgpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeTargetPlatform {
    #[serde(rename = "browser-wasm-webgpu")]
    BrowserWasmWebgpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    #[serde(rename = "uint8")]
    Uint8,
    #[serde(rename = "float32")]
    Float32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineConfigVersion {
    #[serde(rename = "biors.pipeline.v0")]
    BiorsPipelineV0,
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BiorsPackageV0 => "biors.package.v0",
            Self::BiorsPackageV1 => "biors.package.v1",
        })
    }
}

impl fmt::Display for ModelFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Onnx => "onnx",
        })
    }
}

impl fmt::Display for RuntimeBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::OnnxWebgpu => "onnx-webgpu",
        })
    }
}

impl fmt::Display for RuntimeTargetPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BrowserWasmWebgpu => "browser-wasm-webgpu",
        })
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Uint8 => "uint8",
            Self::Float32 => "float32",
        })
    }
}

impl fmt::Display for PipelineConfigVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BiorsPipelineV0 => f.write_str("biors.pipeline.v0"),
        }
    }
}

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Supported package manifest schema versions.
pub enum SchemaVersion {
    #[serde(rename = "biors.package.v0")]
    BiorsPackageV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Supported model artifact formats.
pub enum ModelFormat {
    #[serde(rename = "onnx")]
    Onnx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Supported runtime backends.
pub enum RuntimeBackend {
    #[serde(rename = "onnx-webgpu")]
    OnnxWebgpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Supported runtime target platforms.
pub enum RuntimeTargetPlatform {
    #[serde(rename = "browser-wasm-webgpu")]
    BrowserWasmWebgpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Supported tensor element dtypes.
pub enum DataType {
    #[serde(rename = "uint8")]
    Uint8,
    #[serde(rename = "float32")]
    Float32,
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::BiorsPackageV0 => "biors.package.v0",
        };
        f.write_str(value)
    }
}

impl fmt::Display for ModelFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Onnx => "onnx",
        };
        f.write_str(value)
    }
}

impl fmt::Display for RuntimeBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::OnnxWebgpu => "onnx-webgpu",
        };
        f.write_str(value)
    }
}

impl fmt::Display for RuntimeTargetPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::BrowserWasmWebgpu => "browser-wasm-webgpu",
        };
        f.write_str(value)
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Uint8 => "uint8",
            Self::Float32 => "float32",
        };
        f.write_str(value)
    }
}

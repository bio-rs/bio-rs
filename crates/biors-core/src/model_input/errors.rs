use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors returned by checked model-input builders.
pub enum ModelInputBuildError {
    /// The model input policy is internally invalid.
    InvalidPolicy { message: String },
    /// Workflow provenance received an invalid input hash.
    InvalidInputHash { input_hash: String },
    /// A tokenized sequence has no model input tokens.
    EmptyTokenizedSequence { id: String },
    /// A tokenized sequence still contains unresolved warnings or errors.
    InvalidTokenizedSequence {
        id: String,
        warning_count: usize,
        error_count: usize,
    },
}

impl fmt::Display for ModelInputBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPolicy { message } => write!(f, "invalid model input policy: {message}"),
            Self::InvalidInputHash { input_hash } => write!(
                f,
                "invalid workflow input hash '{input_hash}': expected fnv1a64:<16 lowercase hex>"
            ),
            Self::EmptyTokenizedSequence { id } => write!(
                f,
                "sequence '{id}' is empty and cannot be converted into model input"
            ),
            Self::InvalidTokenizedSequence {
                id,
                warning_count,
                error_count,
            } => write!(
                f,
                "sequence '{id}' is not model-ready: {warning_count} warnings and {error_count} errors must be resolved before building model input"
            ),
        }
    }
}

impl std::error::Error for ModelInputBuildError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors returned when validating already-built model-input payloads.
pub enum ModelInputPayloadError {
    /// One record has different input ID and attention-mask lengths.
    LengthMismatch {
        id: String,
        input_ids: usize,
        attention_mask: usize,
    },
    /// Fixed-length payload records must match the policy max length.
    FixedLengthMismatch {
        id: String,
        expected: usize,
        actual: usize,
    },
    /// No-padding payload records cannot exceed the policy max length.
    NoPaddingLengthExceeded {
        id: String,
        max_length: usize,
        actual: usize,
    },
    /// Attention masks must contain only `0` and `1`.
    NonBinaryAttentionMask { id: String, index: usize, value: u8 },
    /// A record has no tokens selected by its attention mask.
    EmptyUnmaskedTokens { id: String },
}

impl ModelInputPayloadError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::LengthMismatch { .. } => "model_input.length_mismatch",
            Self::FixedLengthMismatch { .. } => "model_input.fixed_length_mismatch",
            Self::NoPaddingLengthExceeded { .. } => "model_input.no_padding_length_exceeded",
            Self::NonBinaryAttentionMask { .. } => "model_input.non_binary_attention_mask",
            Self::EmptyUnmaskedTokens { .. } => "model_input.empty_attention_mask",
        }
    }
}

impl fmt::Display for ModelInputPayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthMismatch {
                id,
                input_ids,
                attention_mask,
            } => write!(
                f,
                "record '{id}' has {input_ids} input ids but {attention_mask} attention-mask values"
            ),
            Self::FixedLengthMismatch {
                id,
                expected,
                actual,
            } => write!(
                f,
                "record '{id}' has {actual} input ids, expected fixed length {expected}"
            ),
            Self::NoPaddingLengthExceeded {
                id,
                max_length,
                actual,
            } => write!(
                f,
                "record '{id}' has {actual} input ids, exceeding max_length {max_length}"
            ),
            Self::NonBinaryAttentionMask { id, index, value } => write!(
                f,
                "record '{id}' attention_mask[{index}] is {value}, expected 0 or 1"
            ),
            Self::EmptyUnmaskedTokens { id } => {
                write!(f, "record '{id}' has no unmasked tokens")
            }
        }
    }
}

impl std::error::Error for ModelInputPayloadError {}

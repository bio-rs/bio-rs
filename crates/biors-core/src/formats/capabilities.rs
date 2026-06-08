mod catalog;

use serde::{Deserialize, Serialize};

use super::records::BioFormat;

/// Implementation state for a biological format in the current release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormatSupportStatus {
    /// Parser and validation contract are executable in the current release.
    Supported,
    /// Requirements are documented, but parser support is not exposed yet.
    ReviewedCandidate,
    /// Explicitly out of scope for the current release line.
    Future,
}

/// Public capability and validation-requirement summary for one format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormatCapability {
    /// Format family.
    pub format: BioFormat,
    /// Current support state.
    pub status: FormatSupportStatus,
    /// Shared or format-specific record contract.
    pub record_contract: String,
    /// Validation requirements that must be met before records become trusted.
    pub validation_requirements: Vec<String>,
    /// Non-contract notes for users and implementers.
    pub notes: Vec<String>,
}

/// Return the current format support matrix.
pub fn format_capabilities() -> Vec<FormatCapability> {
    catalog::format_capabilities()
}

use serde::{Deserialize, Serialize};
use std::fmt;

/// Biological sequence alphabet family used for validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SequenceKind {
    /// Protein sequence validated with the built-in protein-20 policy.
    Protein,
    /// DNA sequence validated with standard and ambiguous IUPAC DNA symbols.
    Dna,
    /// RNA sequence validated with standard and ambiguous IUPAC RNA symbols.
    Rna,
}

impl SequenceKind {
    /// Return the stable alphabet policy name for this kind.
    pub const fn alphabet_name(self) -> &'static str {
        match self {
            Self::Protein => "protein-20",
            Self::Dna => "dna-iupac",
            Self::Rna => "rna-iupac",
        }
    }

    /// Return the display name used in human-readable diagnostics.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Protein => "protein",
            Self::Dna => "DNA",
            Self::Rna => "RNA",
        }
    }
}

impl fmt::Display for SequenceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protein => write!(f, "protein"),
            Self::Dna => write!(f, "dna"),
            Self::Rna => write!(f, "rna"),
        }
    }
}

/// User selection for kind-aware validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SequenceKindSelection {
    /// Detect each sequence kind from its normalized symbols.
    Auto,
    /// Validate with an explicit kind, even if another kind would fit better.
    Explicit(SequenceKind),
}

impl SequenceKindSelection {
    /// Return the explicit kind when detection is disabled.
    pub const fn explicit_kind(self) -> Option<SequenceKind> {
        match self {
            Self::Auto => None,
            Self::Explicit(kind) => Some(kind),
        }
    }
}

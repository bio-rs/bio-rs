use super::parser::SmilesParser;
use crate::molecule::smiles::SmilesParseError;

impl SmilesParser<'_> {
    pub(super) fn unexpected_character(&self, column: usize, character: char) -> SmilesParseError {
        SmilesParseError::UnexpectedCharacter {
            line: self.line,
            column,
            character,
            record_index: self.record_index,
        }
    }

    pub(super) fn dangling_bond(&self, column: usize) -> SmilesParseError {
        SmilesParseError::DanglingBond {
            line: self.line,
            column,
            record_index: self.record_index,
        }
    }

    pub(super) fn invalid_branch(&self, column: usize) -> SmilesParseError {
        SmilesParseError::InvalidBranch {
            line: self.line,
            column,
            record_index: self.record_index,
        }
    }

    pub(super) fn unclosed_branch(&self, column: usize) -> SmilesParseError {
        SmilesParseError::UnclosedBranch {
            line: self.line,
            column,
            record_index: self.record_index,
        }
    }

    pub(super) fn unmatched_branch(&self, column: usize) -> SmilesParseError {
        SmilesParseError::UnmatchedBranch {
            line: self.line,
            column,
            record_index: self.record_index,
        }
    }

    pub(super) fn unclosed_ring(&self, ring: u16, column: usize) -> SmilesParseError {
        SmilesParseError::UnclosedRing {
            line: self.line,
            column,
            ring,
            record_index: self.record_index,
        }
    }

    pub(super) fn invalid_ring_closure(&self, column: usize) -> SmilesParseError {
        SmilesParseError::InvalidRingClosure {
            line: self.line,
            column,
            record_index: self.record_index,
        }
    }

    pub(super) fn invalid_bracket_atom(
        &self,
        column: usize,
        token: &str,
        reason: impl Into<String>,
    ) -> SmilesParseError {
        SmilesParseError::InvalidBracketAtom {
            line: self.line,
            column,
            token: token.to_string(),
            reason: reason.into(),
            record_index: self.record_index,
        }
    }
}

use super::super::{FormatCapability, FormatSupportStatus};
use crate::formats::BioFormat;

pub(super) fn capabilities() -> Vec<FormatCapability> {
    vec![
        FormatCapability {
            format: BioFormat::Csv,
            status: FormatSupportStatus::ReviewedCandidate,
            record_contract: "FormatRecord with header-derived fields".to_string(),
            validation_requirements: vec![
                "header row required with non-empty unique column names".to_string(),
                "RFC-style quoted field handling before biological validation".to_string(),
                "stable delimiter and newline policy".to_string(),
                "caller-selected biological columns for sequence/variant/entity validation"
                    .to_string(),
            ],
            notes: vec![
                "CSV remains reviewed until a dependency-light parser contract is frozen"
                    .to_string(),
            ],
        },
        FormatCapability {
            format: BioFormat::Tsv,
            status: FormatSupportStatus::ReviewedCandidate,
            record_contract: "FormatRecord with header-derived fields".to_string(),
            validation_requirements: vec![
                "header row required with non-empty unique column names".to_string(),
                "tab-delimited row width parity".to_string(),
                "no silent trimming of biological field values".to_string(),
                "caller-selected biological columns for sequence/variant/entity validation"
                    .to_string(),
            ],
            notes: vec![
                "TSV parser can land before CSV if quoted-field support remains deferred"
                    .to_string(),
            ],
        },
    ]
}

use super::super::{FormatCapability, FormatSupportStatus};
use crate::formats::BioFormat;

pub(super) fn capabilities() -> Vec<FormatCapability> {
    vec![
        FormatCapability {
            format: BioFormat::Fasta,
            status: FormatSupportStatus::Supported,
            record_contract: "SequenceRecord / legacy ProteinSequence".to_string(),
            validation_requirements: vec![
                "non-empty header identifiers".to_string(),
                "non-empty sequence body per record".to_string(),
                "protein, DNA, RNA, or auto-detected alphabet validation".to_string(),
                "stable per-record warning/error diagnostics".to_string(),
            ],
            notes: vec!["existing sequence parser and tokenizer input surface".to_string()],
        },
        FormatCapability {
            format: BioFormat::Fastq,
            status: FormatSupportStatus::Supported,
            record_contract: "FastqRecord plus shared FormatRecord projection".to_string(),
            validation_requirements: vec![
                "header line starts with @ and includes a non-empty identifier".to_string(),
                "separator line starts with + and optional identifier matches the header"
                    .to_string(),
                "sequence body is non-empty and validates against DNA IUPAC policy".to_string(),
                "quality string length exactly matches normalized sequence length".to_string(),
                "quality symbols are printable Phred+33 ASCII characters".to_string(),
            ],
            notes: vec![
                "multi-line sequence and quality bodies are accepted".to_string(),
                "quality scores are validated structurally but not decoded into probabilities"
                    .to_string(),
            ],
        },
    ]
}

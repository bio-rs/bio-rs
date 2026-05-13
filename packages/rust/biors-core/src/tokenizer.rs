use crate::error::BioRsError;
use crate::fasta_scan::{scan_fasta_reader, scan_fasta_str};
use crate::verification::StableInputHasher;
use std::io::BufRead;

mod config;
mod lookup;
mod protein;
mod sinks;
mod types;
mod vocab;
pub use config::{
    inspect_protein_tokenizer_config, load_protein_tokenizer_config_json,
    protein_tokenizer_config_for_profile, ProteinTokenizerConfig, ProteinTokenizerInspection,
    ProteinTokenizerProfile, SpecialToken, SpecialTokenSet,
};
pub use protein::{tokenize_protein, tokenize_protein_with_config, ProteinTokenizer, Tokenizer};
use sinks::{SummaryRecordSink, TokenizedRecordSink};
pub use types::{ProteinBatchSummary, SummarizedFastaInput, TokenizedFastaInput, TokenizedProtein};
use vocab::TOKEN_LOOKUP_MISSING;
pub use vocab::{
    load_protein_20_vocab, load_vocab_json, protein_20_unknown_token_policy,
    protein_20_vocab_tokens, protein_20_vocabulary, TokenizerError, UnknownTokenPolicy, VocabToken,
    Vocabulary, PROTEIN_20_UNKNOWN_TOKEN_ID,
};

/// Parse FASTA text and tokenize each protein sequence.
pub fn tokenize_fasta_records(input: &str) -> Result<Vec<TokenizedProtein>, BioRsError> {
    let mut sink = TokenizedRecordSink::default();
    scan_fasta_str(input, &mut sink)?;
    Ok(sink.records)
}

/// Tokenize FASTA records from a buffered reader and include a stable input hash.
pub fn tokenize_fasta_records_reader<R: BufRead>(
    reader: R,
) -> Result<TokenizedFastaInput, crate::error::FastaReadError> {
    tokenize_fasta_records_reader_with_config(
        reader,
        &ProteinTokenizerConfig {
            profile: ProteinTokenizerProfile::Protein20,
            add_special_tokens: false,
        },
    )
}

/// Tokenize FASTA records from a buffered reader using an explicit tokenizer config.
pub fn tokenize_fasta_records_reader_with_config<R: BufRead>(
    reader: R,
    config: &ProteinTokenizerConfig,
) -> Result<TokenizedFastaInput, crate::error::FastaReadError> {
    let mut sink = TokenizedRecordSink::default();
    sink.set_config(config.clone());
    let mut hasher = StableInputHasher::new();
    scan_fasta_reader(reader, &mut sink, |line| hasher.update(line))?;
    Ok(TokenizedFastaInput {
        input_hash: hasher.finalize(),
        records: sink.records,
    })
}

/// Summarize FASTA records from a buffered reader without materializing token vectors.
pub fn summarize_fasta_records_reader<R: BufRead>(
    reader: R,
) -> Result<SummarizedFastaInput, crate::error::FastaReadError> {
    let mut sink = SummaryRecordSink::default();
    let mut hasher = StableInputHasher::new();
    scan_fasta_reader(reader, &mut sink, |line| hasher.update(line))?;

    Ok(SummarizedFastaInput {
        input_hash: hasher.finalize(),
        summary: sink.summary,
    })
}

/// Summarize a slice of tokenized proteins.
pub fn summarize_tokenized_proteins(proteins: &[TokenizedProtein]) -> ProteinBatchSummary {
    ProteinBatchSummary {
        records: proteins.len(),
        total_length: proteins.iter().map(|protein| protein.length).sum(),
        valid_records: proteins.iter().filter(|protein| protein.valid).count(),
        warning_count: proteins.iter().map(|protein| protein.warnings.len()).sum(),
        error_count: proteins.iter().map(|protein| protein.errors.len()).sum(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn tokenize_fasta_records_basic() {
        let input = ">seq1\nACDE\n>seq2\nFGHI\n";
        let result = tokenize_fasta_records(input).expect("valid FASTA");

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "seq1");
        assert_eq!(result[0].tokens, vec![0, 1, 2, 3]);
        assert_eq!(result[0].length, 4);
        assert!(result[0].valid);
        assert_eq!(result[1].id, "seq2");
        assert_eq!(result[1].tokens, vec![4, 5, 6, 7]);
        assert_eq!(result[1].length, 4);
        assert!(result[1].valid);
    }

    #[test]
    fn tokenize_fasta_records_with_issues() {
        let input = ">seq1\nACXDE\n>seq2\nAC*DE\n";
        let result = tokenize_fasta_records(input).expect("valid FASTA");

        assert_eq!(result.len(), 2);
        assert!(!result[0].valid);
        assert_eq!(result[0].warnings.len(), 1);
        assert_eq!(result[0].errors.len(), 0);
        assert_eq!(
            result[0].tokens,
            vec![0, 1, PROTEIN_20_UNKNOWN_TOKEN_ID, 2, 3]
        );
        assert!(!result[1].valid);
        assert_eq!(result[1].warnings.len(), 0);
        assert_eq!(result[1].errors.len(), 1);
        assert_eq!(
            result[1].tokens,
            vec![0, 1, PROTEIN_20_UNKNOWN_TOKEN_ID, 2, 3]
        );
    }

    #[test]
    fn tokenize_fasta_records_reader_basic() {
        let cursor = Cursor::new(b">seq1\nKLMN\n>seq2\nPQRS\n");
        let result = tokenize_fasta_records_reader(cursor).expect("valid FASTA");

        assert!(!result.input_hash.is_empty());
        assert_eq!(result.records.len(), 2);
        assert_eq!(result.records[0].id, "seq1");
        assert_eq!(result.records[0].tokens, vec![8, 9, 10, 11]);
        assert_eq!(result.records[0].alphabet, "protein-20");
        assert!(result.records[0].valid);
        assert_eq!(result.records[1].id, "seq2");
        assert_eq!(result.records[1].tokens, vec![12, 13, 14, 15]);
        assert!(result.records[1].valid);
    }

    #[test]
    fn tokenize_fasta_records_reader_with_config_special_tokens() {
        let config = ProteinTokenizerConfig {
            profile: ProteinTokenizerProfile::Protein20Special,
            add_special_tokens: true,
        };
        let cursor = Cursor::new(b">seq1\nAC\n");
        let result =
            tokenize_fasta_records_reader_with_config(cursor, &config).expect("valid FASTA");

        assert!(!result.input_hash.is_empty());
        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].id, "seq1");
        assert_eq!(result.records[0].length, 4);
        assert_eq!(result.records[0].tokens, vec![22, 0, 1, 23]);
        assert_eq!(result.records[0].alphabet, "protein-20-special");
        assert!(result.records[0].valid);
    }

    #[test]
    fn tokenize_fasta_records_reader_with_config_no_special_tokens() {
        let config = ProteinTokenizerConfig {
            profile: ProteinTokenizerProfile::Protein20,
            add_special_tokens: false,
        };
        let cursor = Cursor::new(b">seq1\nTVWY\n");
        let result =
            tokenize_fasta_records_reader_with_config(cursor, &config).expect("valid FASTA");

        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].tokens, vec![16, 17, 18, 19]);
        assert_eq!(result.records[0].length, 4);
        assert_eq!(result.records[0].alphabet, "protein-20");
        assert!(result.records[0].valid);
    }

    #[test]
    fn summarize_fasta_records_reader_basic() {
        let cursor = Cursor::new(b">seq1\nACDE\nFGHI\n>seq2\nKLMN\n");
        let result = summarize_fasta_records_reader(cursor).expect("valid FASTA");

        assert!(!result.input_hash.is_empty());
        assert_eq!(result.summary.records, 2);
        assert_eq!(result.summary.total_length, 12);
        assert_eq!(result.summary.valid_records, 2);
        assert_eq!(result.summary.warning_count, 0);
        assert_eq!(result.summary.error_count, 0);
    }

    #[test]
    fn summarize_fasta_records_reader_with_issues() {
        let cursor = Cursor::new(b">seq1\nACXDE\n>seq2\nAC*DE\n>seq3\nACDE\n");
        let result = summarize_fasta_records_reader(cursor).expect("valid FASTA");

        assert_eq!(result.summary.records, 3);
        assert_eq!(result.summary.total_length, 14);
        assert_eq!(result.summary.valid_records, 1);
        assert_eq!(result.summary.warning_count, 1);
        assert_eq!(result.summary.error_count, 1);
    }

    #[test]
    fn summarize_tokenized_proteins_batch() {
        let proteins = vec![
            TokenizedProtein {
                id: "seq1".to_string(),
                length: 4,
                alphabet: "protein-20".to_string(),
                valid: true,
                tokens: vec![0, 1, 2, 3],
                warnings: vec![],
                errors: vec![],
            },
            TokenizedProtein {
                id: "seq2".to_string(),
                length: 5,
                alphabet: "protein-20".to_string(),
                valid: false,
                tokens: vec![0, 1, PROTEIN_20_UNKNOWN_TOKEN_ID, 2, 3],
                warnings: vec![crate::sequence::ResidueIssue {
                    residue: 'X',
                    position: 3,
                }],
                errors: vec![],
            },
            TokenizedProtein {
                id: "seq3".to_string(),
                length: 6,
                alphabet: "protein-20".to_string(),
                valid: false,
                tokens: vec![0, 1, PROTEIN_20_UNKNOWN_TOKEN_ID, 2, 3, 4],
                warnings: vec![],
                errors: vec![crate::sequence::ResidueIssue {
                    residue: '*',
                    position: 3,
                }],
            },
        ];

        let summary = summarize_tokenized_proteins(&proteins);
        assert_eq!(summary.records, 3);
        assert_eq!(summary.total_length, 15);
        assert_eq!(summary.valid_records, 1);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.error_count, 1);
    }

    #[test]
    fn summarize_tokenized_proteins_empty() {
        let summary = summarize_tokenized_proteins(&[]);
        assert_eq!(summary.records, 0);
        assert_eq!(summary.total_length, 0);
        assert_eq!(summary.valid_records, 0);
        assert_eq!(summary.warning_count, 0);
        assert_eq!(summary.error_count, 0);
    }
}

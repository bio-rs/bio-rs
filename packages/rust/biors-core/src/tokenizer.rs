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
    protein_20_vocab_tokens, protein_20_vocabulary, UnknownTokenPolicy, VocabToken, Vocabulary,
    PROTEIN_20_UNKNOWN_TOKEN_ID,
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

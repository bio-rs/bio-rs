use biors_core::tokenizer::{summarize_tokenized_proteins, tokenize_fasta_records};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokenized = tokenize_fasta_records(">seq1\nACDE\n")?;
    let summary = summarize_tokenized_proteins(&tokenized);

    println!("records: {}", summary.records);
    println!("first_id: {}", tokenized[0].id);
    println!("first_tokens: {:?}", tokenized[0].tokens);

    Ok(())
}

use biors_core::{
    parse_fasta_records_reader, summarize_fasta_records_reader, tokenize_fasta_records_reader,
    FastaReadError,
};
use serde::Serialize;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::process;

#[derive(Debug, Serialize)]
struct BenchmarkResult {
    mode: String,
    records: usize,
    residues: usize,
    canonical_tokens: usize,
    unknown_tokens: usize,
    warning_count: usize,
    error_count: usize,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mode = args.next().ok_or_else(|| usage("missing mode"))?;
    let input_path = args.next().ok_or_else(|| usage("missing input path"))?;
    if args.next().is_some() {
        return Err(usage("too many arguments"));
    }

    let result = match mode.as_str() {
        "parse" => benchmark_parse(&input_path)?,
        "validate" => benchmark_validate(&input_path)?,
        "tokenize" => benchmark_tokenize(&input_path)?,
        _ => return Err(usage("mode must be one of: parse, validate, tokenize")),
    };

    println!(
        "{}",
        serde_json::to_string(&result).map_err(|error| format!("serialize result: {error}"))?
    );
    Ok(())
}

fn benchmark_parse(input_path: &str) -> Result<BenchmarkResult, String> {
    let records = parse_fasta_records_reader(open_reader(input_path)?)
        .map_err(render_fasta_read_error)?
        .records;
    Ok(BenchmarkResult {
        mode: "parse".to_string(),
        records: records.len(),
        residues: records.iter().map(|record| record.sequence.len()).sum(),
        canonical_tokens: 0,
        unknown_tokens: 0,
        warning_count: 0,
        error_count: 0,
    })
}

fn benchmark_validate(input_path: &str) -> Result<BenchmarkResult, String> {
    let report = summarize_fasta_records_reader(open_reader(input_path)?)
        .map_err(render_fasta_read_error)?
        .summary;
    Ok(BenchmarkResult {
        mode: "validate".to_string(),
        records: report.records,
        residues: report.total_length,
        canonical_tokens: report.total_length - report.warning_count - report.error_count,
        unknown_tokens: report.warning_count + report.error_count,
        warning_count: report.warning_count,
        error_count: report.error_count,
    })
}

fn benchmark_tokenize(input_path: &str) -> Result<BenchmarkResult, String> {
    let records = tokenize_fasta_records_reader(open_reader(input_path)?)
        .map_err(render_fasta_read_error)?
        .records;
    Ok(BenchmarkResult {
        mode: "tokenize".to_string(),
        records: records.len(),
        residues: records.iter().map(|record| record.length).sum(),
        canonical_tokens: records
            .iter()
            .map(|record| record.tokens.len() - record.warnings.len() - record.errors.len())
            .sum(),
        unknown_tokens: records
            .iter()
            .map(|record| record.warnings.len() + record.errors.len())
            .sum(),
        warning_count: records.iter().map(|record| record.warnings.len()).sum(),
        error_count: records.iter().map(|record| record.errors.len()).sum(),
    })
}

fn open_reader(input_path: &str) -> Result<BufReader<File>, String> {
    let file = File::open(input_path)
        .map_err(|error| format!("failed to read '{input_path}': {error}"))?;
    Ok(BufReader::new(file))
}

fn render_fasta_read_error(error: FastaReadError) -> String {
    error.to_string()
}

fn usage(reason: &str) -> String {
    format!("{reason}\nusage: cargo run -p biors-core --example benchmark_fasta -- <parse|validate|tokenize> <input.fasta>")
}

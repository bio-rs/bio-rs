use biors_core::{
    model_input::{ModelInputPolicy, PaddingPolicy},
    parse_fasta_records_reader,
    tokenizer::{protein_tokenizer_config_for_profile, ProteinTokenizerProfile},
    workflow::{prepare_model_input_workflow_with_config, SequenceWorkflowInvocation},
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::io::Cursor;
use std::sync::LazyLock;

const ALPHABET: &[u8] = b"ACDEFGHIKLMNPQRSTVWY";

fn generate_sequence(rng_seed: u64, length: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(length);
    let mut seed = rng_seed;
    for _ in 0..length {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let idx = (seed >> 32) as usize % ALPHABET.len();
        result.push(ALPHABET[idx]);
    }
    result
}

fn generate_fasta(records: usize, avg_length: usize, length_var: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(records * (avg_length + 20));
    for i in 0..records {
        let header = format!(">seq_{i}\n");
        data.extend_from_slice(header.as_bytes());

        let length = if length_var > 0 {
            let var =
                ((i.wrapping_mul(9973)) % (length_var * 2 + 1)) as isize - length_var as isize;
            (avg_length as isize + var).max(10) as usize
        } else {
            avg_length
        };

        let seq = generate_sequence(i as u64, length);
        for chunk in seq.chunks(60) {
            data.extend_from_slice(chunk);
            data.push(b'\n');
        }
    }
    data
}

static HUMAN_PROTEOME_DATA: LazyLock<Vec<u8>> = LazyLock::new(|| generate_fasta(20_659, 554, 500));

static MANY_SHORT_DATA: LazyLock<Vec<u8>> = LazyLock::new(|| generate_fasta(20_000, 48, 0));

static HUMAN_PROTEOME_PARSED: LazyLock<biors_core::fasta::ParsedFastaInput> = LazyLock::new(|| {
    parse_fasta_records_reader(Cursor::new(HUMAN_PROTEOME_DATA.as_slice()))
        .expect("generated human-like FASTA parses")
});

static MANY_SHORT_PARSED: LazyLock<biors_core::fasta::ParsedFastaInput> = LazyLock::new(|| {
    parse_fasta_records_reader(Cursor::new(MANY_SHORT_DATA.as_slice()))
        .expect("generated short FASTA parses")
});

fn workflow_policy() -> ModelInputPolicy {
    ModelInputPolicy {
        max_length: 512,
        pad_token_id: 0,
        padding: PaddingPolicy::FixedLength,
    }
}

fn workflow_invocation(label: &str, records: usize) -> SequenceWorkflowInvocation {
    SequenceWorkflowInvocation {
        command: "criterion workflow workload".to_string(),
        arguments: vec![label.to_string(), format!("records={records}")],
    }
}

fn run_current_workflow(input: &biors_core::fasta::ParsedFastaInput, label: &str) {
    let output = prepare_model_input_workflow_with_config(
        input.input_hash.clone(),
        &input.records,
        workflow_policy(),
        protein_tokenizer_config_for_profile(ProteinTokenizerProfile::Protein20),
        workflow_invocation(label, input.records.len()),
    )
    .expect("generated FASTA is model-ready");
    black_box(output);
}

fn benchmark_workflow_human_proteome_current_cli_path(c: &mut Criterion) {
    let data = &*HUMAN_PROTEOME_DATA;
    let mut group = c.benchmark_group("workflow_human_proteome_current_cli_path");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function(
        BenchmarkId::new("parse_plus_workflow", "20659_records"),
        |b| {
            b.iter(|| {
                let input = parse_fasta_records_reader(Cursor::new(data.as_slice()))
                    .expect("generated FASTA parses");
                run_current_workflow(&input, "human_proteome_parse_plus_workflow");
            })
        },
    );
    group.finish();
}

fn benchmark_workflow_human_proteome_parsed_records(c: &mut Criterion) {
    let input = &*HUMAN_PROTEOME_PARSED;
    let mut group = c.benchmark_group("workflow_human_proteome_parsed_records");
    group.throughput(Throughput::Elements(input.records.len() as u64));
    group.bench_function(BenchmarkId::new("workflow_only", "20659_records"), |b| {
        b.iter(|| run_current_workflow(black_box(input), "human_proteome_workflow_only"))
    });
    group.finish();
}

fn benchmark_workflow_many_short_current_cli_path(c: &mut Criterion) {
    let data = &*MANY_SHORT_DATA;
    let mut group = c.benchmark_group("workflow_many_short_current_cli_path");
    group.throughput(Throughput::Elements(20_000));
    group.bench_function(BenchmarkId::new("parse_plus_workflow", "20000_x_48"), |b| {
        b.iter(|| {
            let input = parse_fasta_records_reader(Cursor::new(data.as_slice()))
                .expect("generated FASTA parses");
            run_current_workflow(&input, "many_short_parse_plus_workflow");
        })
    });
    group.finish();
}

fn benchmark_workflow_many_short_parsed_records(c: &mut Criterion) {
    let input = &*MANY_SHORT_PARSED;
    let mut group = c.benchmark_group("workflow_many_short_parsed_records");
    group.throughput(Throughput::Elements(input.records.len() as u64));
    group.bench_function(BenchmarkId::new("workflow_only", "20000_x_48"), |b| {
        b.iter(|| run_current_workflow(black_box(input), "many_short_workflow_only"))
    });
    group.finish();
}

criterion_group!(
    benches,
    benchmark_workflow_human_proteome_current_cli_path,
    benchmark_workflow_human_proteome_parsed_records,
    benchmark_workflow_many_short_current_cli_path,
    benchmark_workflow_many_short_parsed_records,
);
criterion_main!(benches);

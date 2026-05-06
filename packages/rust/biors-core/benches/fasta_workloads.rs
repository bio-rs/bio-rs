use biors_core::{
    parse_fasta_records_reader, tokenize_fasta_records_reader, validate_fasta_reader,
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

static LARGE_FASTA_DATA: LazyLock<Vec<u8>> = LazyLock::new(|| generate_fasta(180_000, 554, 500));

static MANY_SHORT_DATA: LazyLock<Vec<u8>> = LazyLock::new(|| generate_fasta(20_000, 48, 0));

fn benchmark_parse_human_proteome(c: &mut Criterion) {
    let data = &*HUMAN_PROTEOME_DATA;
    let mut group = c.benchmark_group("parse_human_proteome");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function(BenchmarkId::new("parse", "20659_records"), |b| {
        b.iter(|| {
            let cursor = Cursor::new(data.as_slice());
            let result = parse_fasta_records_reader(cursor).unwrap();
            black_box(result);
        })
    });
    group.finish();
}

fn benchmark_validate_human_proteome(c: &mut Criterion) {
    let data = &*HUMAN_PROTEOME_DATA;
    let mut group = c.benchmark_group("validate_human_proteome");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function(BenchmarkId::new("validate", "20659_records"), |b| {
        b.iter(|| {
            let cursor = Cursor::new(data.as_slice());
            let result = validate_fasta_reader(cursor).unwrap();
            black_box(result);
        })
    });
    group.finish();
}

fn benchmark_tokenize_human_proteome(c: &mut Criterion) {
    let data = &*HUMAN_PROTEOME_DATA;
    let mut group = c.benchmark_group("tokenize_human_proteome");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function(BenchmarkId::new("tokenize", "20659_records"), |b| {
        b.iter(|| {
            let cursor = Cursor::new(data.as_slice());
            let result = tokenize_fasta_records_reader(cursor).unwrap();
            black_box(result);
        })
    });
    group.finish();
}

fn benchmark_parse_large_fasta(c: &mut Criterion) {
    let data = &*LARGE_FASTA_DATA;
    let mut group = c.benchmark_group("parse_large_fasta");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function(BenchmarkId::new("parse", "100MB_plus"), |b| {
        b.iter(|| {
            let cursor = Cursor::new(data.as_slice());
            let result = parse_fasta_records_reader(cursor).unwrap();
            black_box(result);
        })
    });
    group.finish();
}

fn benchmark_validate_large_fasta(c: &mut Criterion) {
    let data = &*LARGE_FASTA_DATA;
    let mut group = c.benchmark_group("validate_large_fasta");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function(BenchmarkId::new("validate", "100MB_plus"), |b| {
        b.iter(|| {
            let cursor = Cursor::new(data.as_slice());
            let result = validate_fasta_reader(cursor).unwrap();
            black_box(result);
        })
    });
    group.finish();
}

fn benchmark_tokenize_large_fasta(c: &mut Criterion) {
    let data = &*LARGE_FASTA_DATA;
    let mut group = c.benchmark_group("tokenize_large_fasta");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function(BenchmarkId::new("tokenize", "100MB_plus"), |b| {
        b.iter(|| {
            let cursor = Cursor::new(data.as_slice());
            let result = tokenize_fasta_records_reader(cursor).unwrap();
            black_box(result);
        })
    });
    group.finish();
}

fn benchmark_parse_many_short_records(c: &mut Criterion) {
    let data = &*MANY_SHORT_DATA;
    let mut group = c.benchmark_group("parse_many_short_records");
    group.throughput(Throughput::Elements(20_000));
    group.bench_function(BenchmarkId::new("parse", "20000_x_48"), |b| {
        b.iter(|| {
            let cursor = Cursor::new(data.as_slice());
            let result = parse_fasta_records_reader(cursor).unwrap();
            black_box(result);
        })
    });
    group.finish();
}

fn benchmark_validate_many_short_records(c: &mut Criterion) {
    let data = &*MANY_SHORT_DATA;
    let mut group = c.benchmark_group("validate_many_short_records");
    group.throughput(Throughput::Elements(20_000));
    group.bench_function(BenchmarkId::new("validate", "20000_x_48"), |b| {
        b.iter(|| {
            let cursor = Cursor::new(data.as_slice());
            let result = validate_fasta_reader(cursor).unwrap();
            black_box(result);
        })
    });
    group.finish();
}

fn benchmark_tokenize_many_short_records(c: &mut Criterion) {
    let data = &*MANY_SHORT_DATA;
    let mut group = c.benchmark_group("tokenize_many_short_records");
    group.throughput(Throughput::Elements(20_000));
    group.bench_function(BenchmarkId::new("tokenize", "20000_x_48"), |b| {
        b.iter(|| {
            let cursor = Cursor::new(data.as_slice());
            let result = tokenize_fasta_records_reader(cursor).unwrap();
            black_box(result);
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    benchmark_parse_human_proteome,
    benchmark_validate_human_proteome,
    benchmark_tokenize_human_proteome,
    benchmark_parse_large_fasta,
    benchmark_validate_large_fasta,
    benchmark_tokenize_large_fasta,
    benchmark_parse_many_short_records,
    benchmark_validate_many_short_records,
    benchmark_tokenize_many_short_records,
);
criterion_main!(benches);

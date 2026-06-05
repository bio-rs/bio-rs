use biors_core::{
    hash::sha256_bytes_digest,
    package::{validate_package_manifest_artifacts, PackageManifest},
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

const MODEL_SIZE_BYTES: usize = 32 * 1024 * 1024;

struct PackageFixture {
    base: PathBuf,
    manifest: PackageManifest,
}

static LARGE_MODEL_PACKAGE: LazyLock<PackageFixture> = LazyLock::new(build_large_model_package);

fn build_large_model_package() -> PackageFixture {
    let base = std::env::temp_dir().join(format!(
        "biors-package-validation-bench-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(base.join("models")).expect("create models dir");
    fs::create_dir_all(base.join("fixtures")).expect("create fixtures dir");
    fs::write(base.join("fixtures/tiny.fasta"), b">seq1\nACDEFG\n").expect("write fixture input");
    fs::write(base.join("fixtures/tiny.output.json"), b"{\"ok\":true}\n")
        .expect("write fixture output");

    let model_bytes = deterministic_bytes(MODEL_SIZE_BYTES);
    fs::write(base.join("models/protein-seed.onnx"), &model_bytes).expect("write model");

    let mut manifest = valid_manifest();
    manifest.model.checksum = Some(sha256_bytes_digest(&model_bytes));

    PackageFixture { base, manifest }
}

fn deterministic_bytes(len: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(len);
    let mut seed = 0x5eed_u64;
    for _ in 0..len {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        bytes.push((seed >> 32) as u8);
    }
    bytes
}

fn valid_manifest() -> PackageManifest {
    serde_json::from_str(
        r#"{
          "schema_version": "biors.package.v0",
          "name": "protein-seed",
          "model": {
            "format": "onnx",
            "path": "models/protein-seed.onnx"
          },
          "preprocessing": [
            {
              "name": "protein_fasta_tokenize",
              "implementation": "biors-core",
              "contract": "protein-20"
            }
          ],
          "postprocessing": [
            {
              "name": "classification_scores",
              "implementation": "python-baseline",
              "contract": "float32-vector"
            }
          ],
          "runtime": {
            "backend": "onnx-webgpu",
            "target": "browser-wasm-webgpu"
          },
          "fixtures": [
            {
              "name": "tiny-protein",
              "input": "fixtures/tiny.fasta",
              "expected_output": "fixtures/tiny.output.json"
            }
          ]
        }"#,
    )
    .expect("valid manifest JSON")
}

fn benchmark_large_model_checksum_validation(c: &mut Criterion) {
    let fixture = &*LARGE_MODEL_PACKAGE;
    let mut group = c.benchmark_group("package_validation_large_model");
    group.throughput(Throughput::Bytes(MODEL_SIZE_BYTES as u64));
    group.bench_function(BenchmarkId::new("validate", "32mb_model_checksum"), |b| {
        b.iter(|| {
            let report = validate_package_manifest_artifacts(
                black_box(&fixture.manifest),
                black_box(Path::new(&fixture.base)),
            );
            assert!(report.valid, "{:?}", report.structured_issues);
            black_box(report);
        })
    });
    group.finish();
}

criterion_group!(benches, benchmark_large_model_checksum_validation);
criterion_main!(benches);

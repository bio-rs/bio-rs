# bio-rs

[![CI](https://github.com/bio-rs/bio-rs/workflows/CI/badge.svg)](https://github.com/bio-rs/bio-rs/actions)
[![Release](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml/badge.svg)](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml)
[![Benchmark](https://img.shields.io/badge/benchmark-regression-blue)](benchmarks/cli_surfaces.md)
[![Contracts](https://img.shields.io/badge/contracts-JSON%20v0-blue)](docs/cli-contract.md)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

bio-rs is a Rust-based bio-AI data/tooling engine.

It validates biological data, prepares model-ready inputs, and emits
reproducible JSON contracts across CLI, Rust, Python, WASM, local services, and
agent tools.

## What It Does

- Validates protein, DNA, RNA, FASTQ, PDB, SMILES, SDF, and MOL2 inputs.
- Builds tokenizer IDs, attention masks, model-input payloads, reports, and
  pipeline locks.
- Checks package manifests, model artifacts, checksums, fixtures, and runtime
  bridge plans.
- Exposes the same local-first contracts through the CLI, Rust, Python, WASM,
  MCP, and local HTTP/service schemas.

See [docs/sequence-kind-support.md](docs/sequence-kind-support.md) before making
broad DNA/RNA support claims. Package skeleton generation and Python/Hugging
Face conversion remain protein-first.

## What It Is Not

- Not a full AI agent.
- Not a hosted SaaS platform, model registry, training framework, or remote
  inference service.
- Not a complete bioinformatics suite.
- No biological data uploads, telemetry, remote storage, or hosted workspace
  behavior by default.

## Quickstart

```bash
cargo install biors --version 0.57.2
biors --version
biors doctor
biors seq validate --kind auto testdata/sequences/multi.fasta
printf '>dna\nACGT\n' | biors workflow --profile dna-iupac --max-length 128 -
biors package validate testdata/protein-package/manifest.json
biors service contract
biors service hosted-boundary
```

More examples: [docs/quickstart.md](docs/quickstart.md)

## Main Docs

- [Installation](docs/install.md)
- [CLI contract](docs/cli-contract.md)
- [JSON schemas](schemas)
- [Protein, DNA, and RNA support](docs/sequence-kind-support.md)
- [Biological formats](docs/formats.md)
- [Structure support](docs/structure.md)
- [Molecule support](docs/molecule.md)
- [Package format and conversion](docs/package-format.md)
- [Pipeline config](docs/pipeline-config.md)
- [Service interface and local HTTP mode](docs/service-interface.md)
- [Rust API](docs/rust-api.md)
- [Python API](docs/python-api.md)
- [WASM API](docs/wasm-api.md)
- [Candle backend](docs/candle-backend.md)
- [Versioning policy](docs/versioning.md)
- [Error codes](docs/error-codes.md)
- [Citation metadata](CITATION.cff)

## Roadmap Boundaries

Future work must stay clearly labeled until implemented. This includes hosted
workflow products, package registries, pretrained model-specific inference,
mmCIF parsing, broader file-format parsers, no-code workflows, and general
chemistry tooling.

## Development

```bash
scripts/check-fast.sh
scripts/check.sh
```

Benchmark artifacts are release regression guards, not universal throughput
claims. Re-render checks live in `scripts/check-benchmark-docs.sh`.

## License

Dual licensed under MIT OR Apache-2.0. If you use bio-rs in research software
or publications, cite the repository and version via [CITATION.cff](CITATION.cff).

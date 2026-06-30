# bio-rs

[![CI](https://github.com/bio-rs/bio-rs/workflows/CI/badge.svg)](https://github.com/bio-rs/bio-rs/actions)
[![Release](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml/badge.svg)](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml)
[![Benchmark](https://img.shields.io/badge/benchmark-regression-blue)](benchmarks/cli_surfaces.md)
[![Contracts](https://img.shields.io/badge/contracts-JSON%20v0-blue)](docs/cli-contract.md)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

bio-rs is an AI-ready biological data I/O, validation, and tokenization engine
for biological researchers and research agents. It prepares model-ready inputs,
checks packages and artifacts, and emits reproducible JSON contracts across
CLI, MCP, Rust, Python, WASM, local services, and backend integration surfaces.

Promoted local workflows run without uploading biological data. No API keys,
tokens, secrets, credentials, or network access are required for validation,
model-input preparation, package checks, local service use, or MCP tool calls.
Defaults are no telemetry, no biological data upload, and no external model
calls.

## What It Does

- Validates protein, DNA, RNA, FASTQ, PDB, SMILES, SDF, and MOL2 inputs.
- Builds tokenizer IDs, attention masks, model-input payloads, reports, and
  pipeline locks.
- Checks package manifests, model artifacts, checksums, fixtures, and runtime
  bridge plans.
- Exposes the same local-first contracts through the CLI, Rust, Python, WASM,
  MCP, and local HTTP/service schemas.

See [docs/sequence-kind-support.md](docs/sequence-kind-support.md) before making
broad DNA/RNA support claims. DNA/RNA package manifest validation and explicit
`package init --tokenizer-config` skeletons are supported, but arbitrary
Python/Hugging Face project conversion remains a protein-tokenizer preview.

## Quickstart

```bash
cargo install biors --version 0.57.4
biors --version
biors doctor
biors seq validate --kind auto testdata/sequences/multi.fasta
printf '>dna\nACGT\n' | biors workflow --profile dna-iupac --max-length 128 -
biors package validate testdata/protein-package/manifest.json
biors service contract
```

More examples: [docs/quickstart.md](docs/quickstart.md)

## Evidence

Committed benchmark reports track release regressions across CLI, Python, WASM,
MCP, and optional backend smoke paths. They are reproducible guardrails, not
universal throughput claims.

Current checked-in examples include:

- CLI DNA/RNA validation on 256 records in about 6 ms:
  [benchmarks/cli_surfaces.md](benchmarks/cli_surfaces.md)
- WASM workflow on 256 records in about 8-10 ms:
  [benchmarks/wasm_bindings.md](benchmarks/wasm_bindings.md)
- MCP doctor request overhead at about 60 us:
  [benchmarks/mcp_server.md](benchmarks/mcp_server.md)

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

## Development

```bash
scripts/check-fast.sh
scripts/check.sh
```

## License

Dual licensed under MIT OR Apache-2.0. If you use bio-rs in research software
or publications, cite the repository and version via [CITATION.cff](CITATION.cff).

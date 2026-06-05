# bio-rs

[![CI](https://github.com/bio-rs/bio-rs/workflows/CI/badge.svg)](https://github.com/bio-rs/bio-rs/actions)
[![Release](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml/badge.svg)](https://github.com/bio-rs/bio-rs/actions/workflows/release.yml)
[![Benchmark](https://img.shields.io/badge/benchmark-UniProt%20FASTA-blue)](benchmarks/fasta_vs_biopython.md)
[![Contracts](https://img.shields.io/badge/contracts-JSON%20v0-blue)](docs/cli-contract.md)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

bio-rs is a local, reproducible input layer for bio-AI tools. It takes raw
biological sequences, structure files, tokenizer profiles, package manifests,
model artifacts, and fixture outputs, then turns them into checked contracts
that can run in CLIs, CI, Python notebooks, browsers, and agent tools.

```txt
raw sequence data + package metadata
  -> validation diagnostics + provenance
  -> tokenizer IDs, model-input tensors, pipeline locks, package checks
  -> JSON contracts for Rust, CLI, Python, WASM, MCP, and service hosts
```

Protein, DNA, and RNA validation, tokenization, model-input generation, workflow
generation, Python/WASM/MCP bindings, package artifact validation, and benchmark
regression guards are supported through explicit sequence-kind profiles. Package
generation and Python/Hugging Face conversion remain protein-first; see the
[sequence-kind support matrix](docs/sequence-kind-support.md) before making
broad full-support claims.

> Status: pre-1.0 CLI and JSON contract stabilization.

## Current boundaries

bio-rs is not a hosted model registry, a model training framework, or a remote
inference service. It does not upload biological data or package artifacts by
default. Current binary archives are limited to the platforms documented in
[docs/install.md](docs/install.md); other platforms should use source builds
until release artifacts are added. Package trust is local validation,
checksums, schemas, and reproducible lockfiles, not a remote signing or review
service.

## Why bio-rs?

Bio research code often starts as a notebook and then has to survive handoff:
to a CLI, a CI job, a browser demo, an agent, a package fixture, or a service
owned by someone else. The brittle part is usually not the model. It is the
input contract around the model: what was parsed, how residues were normalized,
which symbols were warnings, which tokenizer IDs were emitted, whether the
model input is actually safe to run, and which artifacts were used.

bio-rs makes that layer explicit:

- validate protein, DNA, and RNA with stable diagnostics instead of hidden
  string cleanup
- preserve token positions with known profile IDs instead of shortening inputs
  silently
- emit model-input JSON only when unresolved residues have been handled
- pin provenance, vocabulary hashes, output hashes, package checksums, and
  pipeline lockfiles for repeatable runs
- expose the same deterministic core through Rust, the CLI, Python, WASM, MCP,
  and transport-agnostic service schemas

The goal is not to replace Python research workflows. The goal is to give those
workflows a portable contract layer that is easy to inspect, test, cite, and
share.

## Made For Sharing

The project is built for open-source bio-AI work where examples need to be
reproducible and claims need evidence:

- Researchers can share a small FASTA, command, and JSON contract instead of a
  screenshot from a notebook.
- Model/package authors can ship fixture outputs, checksums, citations, model
  cards, and runtime bridge plans next to their artifacts.
- Tool builders can embed the same preprocessing behavior in CLIs, web demos,
  local agents, and internal services without reimplementing the parser.
- Maintainers can point every public claim at schemas, tests, package artifact
  checks, and benchmark regression guards.

## Quickstart

```bash
cargo install biors --version 0.48.0
biors doctor
biors seq validate --kind auto examples/multi.fasta
printf '@read1\nACGN\n+\n!!!!\n' | biors formats validate --format fastq -
printf '>dna\nACGT\n' | biors workflow --profile dna-iupac --max-length 128 -
biors batch validate --kind auto examples/
biors tokenizer inspect --profile protein-20-special
biors package validate examples/protein-package/manifest.json
biors service contract
biors dataset inspect --source uniprot --version 2026_02 --split train examples/
```

Full commands, demos, and install options: [docs/quickstart.md](docs/quickstart.md)

## Proof

bio-rs keeps performance claims tied to reproducible in-repo benchmarks.

Current release posture: no current-version numeric throughput claim is made for
`0.48.0`. The committed benchmark artifact is historical performance evidence
from `biors-core v0.20.0`; rerun and commit a fresh artifact before using these
numbers as evidence for a later release.

The current code includes Criterion regression guards for fixed-length
model-input construction and selected backend smoke paths, but those
implementation changes are not represented by the historical FASTA table below.
The benchmark artifact records which promoted surfaces have committed numeric
coverage and which are explicit non-claims.

### Historical FASTA benchmark reference

Historical FASTA benchmark baseline (recorded on `biors-core v0.20.0`; not current-version performance evidence):

| Dataset | Matched workload | bio-rs core mean | Biopython mean | bio-rs speedup |
|---|---|---:|---:|---:|
| Human proteome | Parse + validation | **0.036s** | 0.584s | **16.09x** |
| Human proteome | Parse + tokenization | **0.061s** | 0.587s | **9.68x** |
| 100MB+ FASTA | Parse + validation | **0.294s** | 3.994s | **13.59x** |
| 100MB+ FASTA | Parse + tokenization | **0.492s** | 4.040s | **8.22x** |
| Many short records | Parse + validation | **0.007s** | 0.204s | **28.35x** |
| Many short records | Parse + tokenization | **0.010s** | 0.205s | **20.54x** |
| Single long sequence | Parse + validation | **0.005s** | 0.176s | **34.48x** |
| Single long sequence | Parse + tokenization | **0.007s** | 0.177s | **26.67x** |

Benchmark details:

- Datasets:
  - UniProt human reference proteome (`UP000005640`, `9606`)
  - 100MB+ large FASTA generated by repeating the same real proteome to isolate large-input throughput
  - 20,000 short 48-residue records generated from the same proteome residue stream
  - one 960,000-residue sequence generated from the same proteome residue stream
- Matched workloads:
  - pure parse
  - parse plus validation
  - parse plus tokenization
- Current best recorded raw throughput:
  - human proteome parse + validation: `315.4M residues/s`, `360.6 MB/s`
  - 100MB+ FASTA parse + validation: `350.8M residues/s`, `401.1 MB/s`
  - human proteome parse + tokenization: `189.0M residues/s`, `216.1 MB/s`
  - 100MB+ FASTA parse + tokenization: `209.7M residues/s`, `239.8 MB/s`
- Benchmark doc: [benchmarks/fasta_vs_biopython.md](benchmarks/fasta_vs_biopython.md)
- Benchmark script: [scripts/benchmark_fasta_vs_biopython.py](scripts/benchmark_fasta_vs_biopython.py)

This benchmark measures `biors-core` directly and excludes CLI startup and JSON
serialization overhead. It is still workload-specific, not a broad claim that
bio-rs is faster than Biopython across every FASTA workload or researcher input
shape. Until the artifact is refreshed for `0.48.0`, the numeric table above
remains a historical reference.

## What Works Today

`biors-core` provides the Rust engine and data contracts. `biors` provides the
CLI surface.

### Sequence handling
- FASTA parsing and normalization with buffered reader APIs
- FASTQ parsing and validation with shared format metadata, sequence/quality
  length checks, Phred+33 quality character validation, and DNA IUPAC sequence
  diagnostics
- Protein/DNA/RNA validation with per-record kind detection (`--kind auto`)
- Line and record-index diagnostics with residue warning/error reporting

### Structure handling
- PDB fixed-column ATOM/HETATM parsing with `StructureRecord`, `Chain`,
  `Residue3D`, `Atom`, and `Coordinate` contracts
- Chain extraction, `REMARK 465` missing-residue preservation, coordinate
  validation, occupancy checks, and missing element warnings
- Coordinate-derived protein sequence extraction and SEQRES mapping through
  `biors structure validate --format pdb` and
  `biors structure sequence --format pdb`
- mmCIF is reviewed as the next structure parser candidate but is not exposed
  as executable parser support yet

### Tokenization
- `protein-20` tokenization with stable IDs
- `protein-20-special` tokenization with UNK/PAD/CLS/SEP/MASK special tokens
- `dna-iupac` and `rna-iupac` tokenization with stable canonical base IDs
- `dna-iupac-special` and `rna-iupac-special` tokenization with UNK/PAD/CLS/SEP/MASK special tokens
- JSON tokenizer config loading and inspection
- Hugging Face tokenizer config conversion
- Positional token alignment preserved with explicit unknown-token IDs

### Model input
- `model-input` CLI: profile-aware `input_ids`, `attention_mask`, and truncation metadata for protein, DNA, and RNA token profiles
- `workflow` CLI: profile-aware validation → tokenization → model input with readiness issues and reproducibility provenance
- `pipeline` CLI: no-config validate → tokenize → export, or config-driven (TOML/JSON) workflows with lockfile generation
- `debug` CLI: step-by-step per-record inspection with compact residue markers
- Checked and unchecked model-input builders with safety checks for unresolved residues
- Python, WASM, MCP, package artifact validation, and regression benchmarks cover nucleotide model-ready workflows. Package skeleton/conversion helpers remain protein-first; see [Protein, DNA, and RNA support](docs/sequence-kind-support.md).

### Batch and dataset operations
- `batch validate`: multiple files, recursive directories, quoted globs
- `dataset inspect`: dataset descriptors, sample mapping, file SHA-256 provenance
- `cache inspect` and guarded `cache clean` for local artifact store

### Package management
- Manifest inspection, validation, and migration (v0 → v1)
- Schema compatibility checks and canonical diffs
- SHA-256 checksum verification and fixture verification
- Python project to bio-rs package skeleton conversion
- Runtime bridge planning reports, backend execution abstraction contracts, and
  guarded external-process backend adapters
- Optional Candle backend crate for CPU safetensors linear-probe inference
- Model artifact metadata and runtime/model compatibility checks in package
  bridge reports
- Transport-agnostic service interface contract for service hosts, without
  bundling a server runtime
- Typed validation issue codes and manifest enums

### External interfaces
- `biors-python`: PyO3 bindings for Python integration and notebook workflows
- `biors-wasm`: WebAssembly/JavaScript bindings with TypeScript definitions
- `biors-mcp-server`: local MCP server crate for agent-callable sequence tools
- `service contract`: offline JSON route/schema contract for caller-owned
  service hosts

### Utilities
- `diff`: canonical JSON/raw comparison with SHA-256 hashes
- `doctor`: core CLI, WASM, Python, package, release, and benchmark readiness
- `completions`: shell completion generation
- JSON success/error envelopes for all commands

## Documentation

- [Quickstart](docs/quickstart.md) — install and first commands
- [Installation and distribution](docs/install.md) — cargo, binaries, completions
- [CLI contract](docs/cli-contract.md) — commands, JSON envelopes, exit codes
- [Package format](docs/package-format.md) — manifest layout and research metadata
- [Package conversion](docs/package-conversion.md) — HF/Python project conversion path
- [Candle backend](docs/candle-backend.md) — optional Candle runtime crate
- [Service interface](docs/service-interface.md) — service-host contract and runtime boundary
- [Protein, DNA, and RNA support](docs/sequence-kind-support.md) — public support matrix by surface
- [Pipeline config](docs/pipeline-config.md) — config-driven static preprocessing workflows
- [Biological format support](docs/formats.md) — FASTQ/PDB support and reviewed candidate requirements for GFF3/GTF/BED/VCF/GenBank/UniProt/mmCIF/table formats
- [Structure support](docs/structure.md) — PDB validation, chain extraction, sequence mapping, and mmCIF candidate requirements
- [Error code registry](docs/error-codes.md)
- [Rust API](docs/rust-api.md)
- [Python API](docs/python-api.md)
- [WASM API](docs/wasm-api.md)
- [Versioning policy](docs/versioning.md)
- [JSON schemas](schemas)
- [Citation metadata](CITATION.cff)

## Not yet

These are roadmap directions, not current capabilities:

- hosted web workflows
- pretrained model-specific inference backends
- package registry or plugin ecosystem
- general-purpose chemistry tooling
- mmCIF structure parsing beyond reviewed candidate requirements
- no-code or low-code workflows

## Development

Run checks:

```bash
scripts/check.sh
```

Run the faster local commit gate:

```bash
scripts/check-fast.sh
```

The check suite runs:

- `cargo fmt`
- shell and Python syntax checks for repo scripts
- benchmark Markdown regeneration check
- release workflow publish-order invariant check
- Rust checks
- `biors-core` `wasm32-unknown-unknown` build check
- tests
- `cargo clippy` with warnings denied

Reproduce the FASTA benchmark:

```bash
cargo build --release -p biors-core --example benchmark_fasta
python3 -m venv .venv-bench
. .venv-bench/bin/activate
pip install biopython
python scripts/benchmark_fasta_vs_biopython.py
cat benchmarks/fasta_vs_biopython.json
```

The benchmark script updates both `benchmarks/fasta_vs_biopython.json` and
`benchmarks/fasta_vs_biopython.md`. `scripts/check-benchmark-docs.sh` verifies
that the Markdown report still matches the JSON artifact.

Compare two benchmark artifacts:

```bash
python scripts/compare-benchmark-artifacts.py before.json after.json
```

Run the Rust library example:

```bash
cargo run -p biors-core --example tokenize
```

## Repository Map

Most users only need the `biors` CLI or one binding package. This map is for
contributors and integrators who need to understand where public surfaces live.

```txt
packages/
  rust/
    biors/                 CLI
    biors-backend-candle/  Optional Candle runtime backend
    biors-core/            Core engine + contracts
    biors-mcp-server/      Local MCP server
    biors-python/          PyO3 bindings
    biors-wasm/            WASM/JS bindings

schemas/
  JSON contracts for CLI, package, pipeline, service, tokenizer, and workflow outputs

examples/
  protein.fasta
  multi.fasta
  model-input-contract/
    protein.fasta
    protein-20-special.config.json
    protein-20-special.expected.json
    reference-python-parity.json
  python/
    esm_from_biors_json.py
    pandas_numpy_friendly.py
    protbert_from_biors_json.py
    reference_preprocess.py
  protein-package/
    models/
    docs/
    manifest.json
    observations.json
    fixtures/
    observed/
    tokenizers/
    vocabs/
    pipelines/
  pipeline/
    protein.toml
    protein.yaml
    protein.json
    pipeline.lock
```

The full schema inventory and command-to-schema mapping live in
[docs/cli-contract.md](docs/cli-contract.md). Release tests keep that contract
inventory aligned with the files under [schemas](schemas).

## Protein-20 alphabet

```txt
A C D E F G H I K L M N P Q R S T V W Y
```

Token IDs follow that order, starting at `0`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for local setup, checks, and PR expectations.

## License

Dual licensed under MIT OR Apache-2.0. If you use bio-rs in research software
or publications, cite the repository and version via [CITATION.cff](CITATION.cff).

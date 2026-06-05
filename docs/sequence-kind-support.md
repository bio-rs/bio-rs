# Protein, DNA, And RNA Support Matrix

This matrix is the public capability boundary for sequence-kind support. A
surface is listed as supported only when the implementation, tests, schemas or
typed declarations, docs, and benchmark/non-claim boundaries are aligned.
The surface/status cells are checked against
`contracts/sequence-kind-support.json` by
`scripts/check-sequence-kind-support-docs.py`.

| Surface | Protein | DNA | RNA | Notes |
|---|---|---|---|---|
| CLI `fasta validate` / `seq validate` | Supported | Supported | Supported | `--kind auto` can classify per record; explicit `--kind` applies one policy to all records. |
| CLI `batch validate` | Supported | Supported | Supported | Uses the same kind policies as sequence validation and records aggregate `kind_counts`. |
| CLI `tokenize` | Supported | Supported | Supported | Use explicit profiles: `protein-20`, `protein-20-special`, `dna-iupac`, `dna-iupac-special`, `rna-iupac`, `rna-iupac-special`. |
| CLI `model-input` | Supported | Supported | Supported | Requires a profile matching the biological sequence kind and rejects unresolved warnings/errors before emitting model-ready arrays. |
| CLI `workflow` | Supported | Supported | Supported | Emits validation, tokenization, model-input readiness, provenance, vocabulary hash, and output-content hash for the selected profile. |
| CLI `pipeline` no-config mode | Supported | Use `workflow` or config mode | Use `workflow` or config mode | Legacy no-config `pipeline` keeps the protein default for backward compatibility. |
| CLI `pipeline --config` | Supported | Supported | Supported | `validate.kind` must match `tokenize.profile`; mismatches fail before input execution. |
| Rust `biors-core` validation | Supported | Supported | Supported | Kind-aware validation and auto-detection live in the `sequence` module. |
| Rust `biors-core` tokenization/model-input/workflow | Supported | Supported | Supported | Public type names still include legacy protein terminology, but built-in profiles are sequence-kind aware. |
| Python bindings | Supported | Supported | Supported | `validate_fasta_input_with_kind`, tokenization, checked model-input building, and workflow helpers accept explicit kinds/profiles. `validate_fasta_input` remains protein-default for compatibility. |
| WASM / JavaScript bindings | Supported | Supported | Supported | `validateFasta`, `tokenize`, and `runWorkflow` accept nucleotide kinds/profiles. |
| MCP server | Supported | Supported | Supported | `validate`, `tokenize`, and `workflow` accept nucleotide kinds/profiles and reject kind/profile mismatches. |
| Service contract schemas | Supported | Supported | Supported | The offline service request schemas enumerate the same profiles; service hosts own transport and route execution. |
| Package manifest validation | Supported | Supported | Supported | Tokenizer config and vocab artifacts are validated for built-in protein, DNA, and RNA profile contracts. `package init --tokenizer-config` generates a matching kind/profile pipeline. |
| Package conversion from Python/HF projects | Supported for protein preview configs | Not promoted | Not promoted | `convert-hf` and `convert-project` remain protein-tokenizer preview tools until nucleotide project-conversion fixtures exist. |
| Benchmarks | Supported | Supported | Supported | Current nucleotide numbers are committed regression guards only, not public throughput claims. |

Do not describe DNA/RNA as "full support" without naming the remaining
protein-first project-conversion limitations above. The supported model-ready
nucleotide path today is explicit profile tokenization, model-input generation,
workflow output, binding parity, MCP parity, package artifact validation, and
benchmark regression coverage.

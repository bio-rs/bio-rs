# Reliability And Input Safety

bio-rs treats FASTA, JSON manifests, package observations, and fixture paths as
untrusted local input.

## CLI UX Policy

- Successful commands write JSON to stdout and keep stderr empty.
- `--json` errors write JSON to stdout and keep stderr empty.
- Human-readable errors write to stderr as `error[code]: message`.
- User input failures exit with code `2`.
- I/O and internal serialization failures exit with code `1`.
- Command help should list every stable top-level command and the global
  `--json` flag.

## Malformed Input Guarantee

Malformed FASTA, invalid UTF-8, invalid JSON, bad package paths, checksum
mismatches, and non-model-ready residues must return a `CliError`; they must not
panic or produce partial success output.

The guarantee is enforced by CLI tests that cover:

- missing FASTA headers
- empty FASTA identifiers
- invalid UTF-8 sequence input
- malformed JSON package input
- manifest path traversal
- model-input policy failures
- model-input residue failures

## Malicious Input Policy

bio-rs rejects absolute package asset paths and `..` traversal so package
manifests and fixture observations remain rooted in their package directory.

bio-rs does not execute model files, tokenizer files, vocab files, or fixture
payloads during validation. Package validation reads files only to verify
presence and checksum contracts.

## Large File Handling

FASTA-backed CLI paths use buffered reader APIs. `inspect` uses a summary path
that avoids materializing token vectors. `tokenize`, `seq validate`, and
`model-input` stream parsing from the reader but retain their public JSON output
payloads before serialization.

Use the large-file benchmark helper after building the release binary:

```bash
cargo build --release -p biors
python3 scripts/benchmark_large_file_streaming.py
```

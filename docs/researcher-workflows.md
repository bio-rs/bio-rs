# Researcher Workflows

These recipes define the promoted local 1.0 workflows for biological
researchers and research agents. They use repository fixtures, produce local
JSON or local files, and avoid registry publish, external model calls, private
data, and network dependencies. No API keys, tokens, secrets, credentials, or
network access are required for these recipes, and the commands run without
uploading biological data.

Run the executable checks from a source checkout:

```bash
scripts/check-researcher-workflows.sh --all
```

Use `BIORS_BIN=/path/to/biors` to check an installed binary. Without
`BIORS_BIN`, the script uses `target/debug/biors` and builds it when missing.

## Recipes

### validate-fasta-fastq

- Command/tool:
  - `biors fasta validate testdata/researcher-workflows/protein.fasta`
  - `biors formats validate --format fastq testdata/researcher-workflows/reads.fastq`
- Fixture input:
  - `testdata/researcher-workflows/protein.fasta`
  - `testdata/researcher-workflows/reads.fastq`
- Expected output shape:
  - FASTA returns a CLI success envelope whose `data.records` is `1`.
  - FASTQ returns a CLI success envelope with `data.format` set to `fastq`.
- Failure case:
  - `biors fasta validate testdata/researcher-workflows/invalid.fasta` exits
    non-zero with a stable FASTA error code, so the next action is to repair
    the missing header before running downstream workflows. In JSON mode the
    error includes `recovery_hint`.

### validate-sequence-kinds

- Command/tool:
  - `biors seq validate --kind protein testdata/researcher-workflows/protein.fasta`
  - `biors seq validate --kind dna testdata/researcher-workflows/dna.fasta`
  - `biors seq validate --kind rna testdata/researcher-workflows/rna.fasta`
- Fixture input:
  - `testdata/researcher-workflows/protein.fasta`
  - `testdata/researcher-workflows/dna.fasta`
  - `testdata/researcher-workflows/rna.fasta`
- Expected output shape:
  - Each command returns a CLI success envelope with `data.records` set to `1`
    and kind-specific validation counts.
- Failure case:
  - A kind/profile mismatch should stop before model-ready output; choose a
    matching sequence kind and tokenizer profile, then rerun the command. The
    workflow output keeps `model_ready=false` and records the next action in
    `readiness_issues[*].recovery_hint`.

### protein-model-ready-workflow

- Command/tool:
  - `biors tokenize --profile protein-20 testdata/researcher-workflows/protein.fasta`
  - `biors model-input --max-length 16 testdata/researcher-workflows/protein.fasta`
  - `biors workflow --max-length 16 testdata/researcher-workflows/protein.fasta`
- Fixture input:
  - `testdata/researcher-workflows/protein.fasta`
- Expected output shape:
  - Tokenize returns token records.
  - Model-input returns `data.records`.
  - Workflow returns a sequence workflow envelope with provenance and
    model-input payloads.
- Failure case:
  - If a protein input has invalid FASTA structure or unsupported residues, fix
    the input or choose the matching tokenizer profile before passing the
    record to package or report workflows. JSON errors include
    `recovery_hint` for model-input failures.

### invalid-workflow-recovery

- Command/tool:
  - `biors --json workflow --max-length 16 testdata/researcher-workflows/invalid.fasta`
- Fixture input:
  - `testdata/researcher-workflows/invalid.fasta`
- Expected output shape:
  - The command exits non-zero and emits a JSON error envelope.
- Failure case:
  - The next action is to add a FASTA header line and rerun
    `fasta validate`, then retry `workflow` only after validation passes.

### molecule-structure-validation

- Command/tool:
  - `biors molecule validate --format smiles testdata/researcher-workflows/molecule.smi`
  - `biors structure validate --format pdb testdata/researcher-workflows/structure.pdb`
- Fixture input:
  - `testdata/researcher-workflows/molecule.smi`
  - `testdata/researcher-workflows/structure.pdb`
- Expected output shape:
  - Molecule validation returns `data.format` set to `smiles`.
  - Structure validation returns `data.format` set to `pdb`.
- Failure case:
  - Parser error codes such as `smiles.*` or `pdb.*` indicate the record should
    be repaired or routed to a specialized domain parser before analysis.

### package-validate-verify-bridge

- Command/tool:
  - `biors package inspect testdata/protein-package/manifest.json`
  - `biors package validate testdata/protein-package/manifest.json`
  - `biors package verify testdata/protein-package/manifest.json testdata/protein-package/observations.json`
  - `biors package bridge testdata/protein-package/manifest.json`
- Fixture input:
  - `testdata/protein-package/manifest.json`
  - `testdata/protein-package/observations.json`
- Expected output shape:
  - Inspect and validate return package metadata and validation reports.
  - Verify returns observation verification status.
  - Bridge returns a runtime bridge report.
- Failure case:
  - Absolute paths, `..` traversal, checksum mismatch, unknown schema version,
    missing files, unsupported public package runtime, or fixture observation
    mismatch should be fixed in the package manifest or observed outputs before
    release-artifact QA. JSON errors include `recovery_hint` where the next
    local action is deterministic.
  - `package bridge` is contract planning only: use `contract_ready`,
    `artifact_checked`, and `execution_ready` instead of treating the legacy
    `ready` alias as execution readiness.

### local-report-json-output

- Command/tool:
  - `biors workflow --max-length 16 testdata/researcher-workflows/protein.fasta > workflow.json`
  - `biors report generate workflow.json --output workflow-report.md --shareable-json workflow-report.json`
- Fixture input:
  - `testdata/researcher-workflows/protein.fasta`
- Expected output shape:
  - `workflow.json` is the local workflow JSON.
  - `workflow-report.md` is deterministic Markdown.
  - `workflow-report.json` is shareable local JSON.
- Failure case:
  - If report generation rejects the input JSON, regenerate the source JSON
    from a supported bio-rs command rather than editing the report by hand.

### mcp-agent-sequence

- Command/tool:
  - MCP tool `validate`
  - MCP tool `workflow`
  - MCP tool `package_validate_fields`
  - MCP tool `package_validate`
- Fixture input:
  - FASTA payload equivalent to `testdata/researcher-workflows/protein.fasta`
  - Package payload equivalent to `testdata/protein-package/manifest.json`
- Expected output shape:
  - The agent calls `validate`, then `workflow`, then package validation tools,
    reading compact JSON results between steps.
- Failure case:
  - Tool errors are treated as structured local failures. The agent should fix
    the local input or manifest, then rerun the failed tool before composing
    downstream work.

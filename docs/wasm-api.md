# WASM API

`biors-wasm` exposes a browser-safe and Node.js-safe subset of
`biors-core` as a WebAssembly module. It supports FASTA parsing, sequence
validation, profile-aware protein/DNA/RNA tokenization, model-input
construction, and the standard preprocessing workflow.

> **Status:** The `biors-wasm` crate is implemented in this repository. Tag
> releases build, test, and publish the npm package through npm trusted
> publishing.

## Installation

```bash
npm install @bio-rs/biors-wasm
```

The package contains the `.wasm` binary, JS glue, and TypeScript declarations.
No hosted service or external runtime is required.

## Module Loading

Import the named exports from the package entrypoint. The bundler-target
package initializes its generated WASM glue when the module is loaded, so there
is no separate default `init` export.

```javascript
import { parseFasta } from "@bio-rs/biors-wasm";

const bytes = new TextEncoder().encode(">seq1\nACDEFGHIKLMNPQRSTVWY\n");
const records = parseFasta(bytes);
console.log(records[0].id);
```

Bundlers such as Vite, Webpack, and Rollup usually resolve the `.wasm` asset
through the generated package glue. Node.js and custom test runners must support
ES module WASM imports for direct package loading.

## API Reference

### `parseFasta(bytes: Uint8Array): FastaRecord[]`

Parses UTF-8 FASTA bytes into records.

```javascript
const records = parseFasta(
  new TextEncoder().encode(">sp|P01308|INS_HUMAN\nMALWMRLLPLLALLALWGPDPAAA\n")
);
```

Returned records contain:

```typescript
interface FastaRecord {
  id: string;
  sequence: string;
}
```

### `validateFasta(bytes: Uint8Array, kind: string): ValidationReport`

Validates FASTA bytes for `"protein"`, `"dna"`, `"rna"`, or `"auto"`.

```javascript
const report = validateFasta(bytes, "protein");
console.log(report.valid_records, report.error_count);
```

The report shape matches the core sequence validation schema:

```typescript
interface SequenceValidationIssue {
  symbol: string;
  position: number;
  kind: "protein" | "dna" | "rna";
  code: "ambiguous_symbol" | "invalid_symbol";
  message: string;
}

interface ValidatedSequence {
  id: string;
  sequence: string;
  kind: "protein" | "dna" | "rna";
  alphabet: string;
  valid: boolean;
  warnings: SequenceValidationIssue[];
  errors: SequenceValidationIssue[];
}

interface ValidationReport {
  records: number;
  valid_records: number;
  warning_count: number;
  error_count: number;
  kind_counts: {
    protein: number;
    dna: number;
    rna: number;
  };
  sequences: ValidatedSequence[];
}
```

### `tokenize(records: FastaRecord[], profile: string): TokenizeOutput`

Tokenizes parsed FASTA records with `"protein-20"`, `"protein-20-special"`,
`"dna-iupac"`, `"dna-iupac-special"`, `"rna-iupac"`, or
`"rna-iupac-special"`.

```javascript
const tokenized = tokenize(records, "protein-20");
console.log(tokenized.records[0].tokens);
```

```typescript
interface ResidueIssue {
  residue: string;
  position: number;
}

interface TokenizedRecord {
  id: string;
  tokens: number[];
  length: number;
  alphabet: string;
  valid: boolean;
  warnings: ResidueIssue[];
  errors: ResidueIssue[];
}

interface TokenizeOutput {
  inputIds: number[][];
  attentionMask: number[][];
  ids: string[];
  records: TokenizedRecord[];
}
```

### `buildModelInput(tokenized: TokenizedRecord[], maxLength: number): ModelInputOutput`

Builds model-ready records with the compatibility default of no padding.

```javascript
const modelInput = buildModelInput(tokenized.records, 512);
console.log(modelInput.records[0].input_ids);
```

### `buildModelInputWithPolicy(tokenized, maxLength, padTokenId, padding): ModelInputOutput`

Builds model-ready records with an explicit padding policy.

```javascript
const modelInput = buildModelInputWithPolicy(
  tokenized.records,
  512,
  0,
  "fixed_length"
);
```

`padding` must be `"no_padding"` or `"fixed_length"`.

```typescript
interface ModelInputRecord {
  id: string;
  input_ids: number[];
  attention_mask: number[];
  truncated: boolean;
}

interface ModelInputOutput {
  policy: {
    max_length: number;
    pad_token_id: number;
    padding: "fixed_length" | "no_padding";
  };
  records: ModelInputRecord[];
}
```

### `runWorkflow(config: WorkflowConfig): WorkflowOutput`

Runs validation, tokenization, model-input generation, and reproducibility
metadata generation in one call.

```javascript
const workflow = runWorkflow({
  fastaBytes: bytes,
  maxLength: 512,
  padding: "fixed_length",
  padTokenId: 0,
});

if (workflow.model_ready && workflow.model_input) {
  console.log(workflow.model_input.records[0].input_ids);
}

console.log(workflow.provenance.input_hash);
```

```typescript
interface WorkflowConfig {
  fastaBytes: Uint8Array;
  kind?: "auto" | "protein" | "dna" | "rna";
  profile?:
    | "protein-20"
    | "protein-20-special"
    | "dna-iupac"
    | "dna-iupac-special"
    | "rna-iupac"
    | "rna-iupac-special";
  maxLength: number;
  padding?: "fixed_length" | "no_padding";
  padTokenId?: number;
}

interface WorkflowReadinessIssue {
  code: string;
  id: string;
  warning_count: number;
  error_count: number;
  message: string;
}

interface WorkflowOutput {
  workflow: string;
  model_ready: boolean;
  validation: ValidationReport;
  tokenization: {
    summary: {
      records: number;
      total_length: number;
      valid_records: number;
      warning_count: number;
      error_count: number;
    };
    records: TokenizedRecord[];
  };
  model_input: ModelInputOutput | null;
  readiness_issues: WorkflowReadinessIssue[];
  provenance: {
    biors_core_version: string;
    invocation: {
      command: string;
      arguments: string[];
    };
    input_hash: string;
    normalization: string;
    validation_alphabet: string;
    tokenizer: {
      name: string;
      vocab_size: number;
      unknown_token_id: number;
      unknown_token_policy: string;
    };
    model_input_policy: {
      max_length: number;
      pad_token_id: number;
      padding: "fixed_length" | "no_padding";
    };
    hashes: {
      vocabulary_sha256: string;
      output_data_sha256: string;
    };
  };
}
```

`runWorkflow` defaults to `protein-20`. Pass `kind` and `profile` when running
DNA or RNA workflows. Explicit kind/profile mismatches are rejected before a
workflow payload is emitted.

## Complete Example

```javascript
import {
  parseFasta,
  validateFasta,
  tokenize,
  buildModelInputWithPolicy,
  runWorkflow,
} from "@bio-rs/biors-wasm";

const fastaText = `>sp|P01308|INS_HUMAN
MALWMRLLPLLALLALWGPDPAAA
>sp|P68871|HBB_HUMAN
MVHLTPEEKSAVTALWGKVNVDEVGGEALGR
`;
const bytes = new TextEncoder().encode(fastaText);

const records = parseFasta(bytes);
const validation = validateFasta(bytes, "protein");

if (validation.error_count > 0) {
  throw new Error(`FASTA has ${validation.error_count} validation errors`);
}

const tokenized = tokenize(records, "protein-20");
const modelInput = buildModelInputWithPolicy(
  tokenized.records,
  512,
  0,
  "fixed_length"
);

const workflow = runWorkflow({
  fastaBytes: bytes,
  maxLength: 512,
  padding: "fixed_length",
  padTokenId: 0,
});

console.log({
  ids: tokenized.ids,
  firstTokens: tokenized.records[0]?.tokens,
  firstInputLength: modelInput.records[0]?.input_ids.length,
  modelReady: workflow.model_ready,
  inputHash: workflow.provenance.input_hash,
});
```

## Boundary

The WASM package does not currently export package-manifest validation or
runtime bridge planning helpers. Use the native Rust crate, CLI, or Python JSON
helpers for those schema-rich reports.

## Related Documents

- [WASM Readiness](wasm-readiness.md)
- [Rust API](rust-api.md)

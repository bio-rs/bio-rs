# WASM API

The `@bio-rs/core-wasm` package exposes a browser-safe subset of `biors-core` as a WebAssembly module. You can parse FASTA, validate sequences, tokenize proteins, build model inputs, inspect package manifests, and plan runtime bridges, all without leaving the browser.

> **Status:** The `biors-wasm` crate and `@bio-rs/biors-wasm` npm package were implemented in v0.45.0. `biors-core` continues to compile for `wasm32-unknown-unknown`, and the WASM binding layer is now available for browser and Node.js consumers.

---

## Installation

```bash
npm install @bio-rs/core-wasm
```

The package contains the compiled `.wasm` binary, JS glue, and TypeScript definitions. No external runtime dependencies are required.

---

## Initialization

Every application must call the async `init` function once before using any other API. This fetches and instantiates the WebAssembly module.

```javascript
import init, { parseFasta, validateFasta } from '@bio-rs/core-wasm';

async function main() {
  await init();

  const records = parseFasta('>seq1\nACDEFGHIKLMNPQRSTVWY\n');
  console.log(records);
}

main();
```

In bundlers like Vite, Webpack, or Rollup, `init` resolves the `.wasm` file automatically through the package's JS glue. In Node.js or test runners, you may need to pass an explicit path or buffer.

---

## API Reference

### `fasta`

Parse and validate FASTA text.

#### `parseFasta(fastaText: string): ProteinSequence[]`

Parse a FASTA string into an array of protein sequence records. Returns an empty array when the input is empty or contains no valid records. Throws `BioRsError` on malformed FASTA that cannot be structurally parsed.

```javascript
const records = parseFasta(
  '>sp|P12345|AATM_RABIT\nACDEFGHIKLMNPQRSTVWY\n>sp|P67890|AATM_MOUSE\nMKWLLKGLA\n'
);
```

#### `validateFasta(fastaText: string, kind: string): ValidationReport`

Validate FASTA text for a specific sequence kind. `kind` accepts `"protein"`, `"dna"`, `"rna"`, or `"auto"`. The report contains per-record diagnostics, residue-level issues, and a top-level `valid` boolean.

```javascript
const report = validateFasta('>seq1\nACDEFGHIKLMNPQRSTVWY\n', 'protein');
console.log(report.valid);   // true
console.log(report.records); // per-record details
```

---

### `tokenizer`

Tokenize protein sequences into stable integer IDs.

#### `tokenize(records: ProteinSequence[], profile: string): TokenizedRecord[]`

Tokenize an array of parsed protein sequences using a named profile. Built-in profiles include `"protein-20"` and `"protein-20-special"`.

```javascript
const tokenized = tokenize(records, 'protein-20-special');
console.log(tokenized[0].tokens); // [2, 3, 4, ...]
```

The `"protein-20-special"` profile emits special tokens for CLS, SEP, PAD, MASK, and UNK. Token IDs are stable across versions for the same profile name.

---

### `modelInput`

Build padded, truncated, model-ready inputs from tokenized records.

#### `buildModelInput(tokenized: TokenizedRecord[], maxLength: number): ModelInput`

Convert tokenized records into a batch-ready `ModelInput` object with `input_ids`, `attention_mask`, and truncation metadata. Sequences longer than `maxLength` are truncated from the right. Shorter sequences are padded to `maxLength` using the profile's PAD token.

```javascript
const modelInput = buildModelInput(tokenized, 512);
console.log(modelInput.records[0].input_ids);
console.log(modelInput.records[0].attention_mask);
```

---

### `workflow`

Run end-to-end validation, tokenization, and model-input preparation in a single call.

#### `runWorkflow(config: WorkflowConfig): WorkflowOutput`

Execute the standard protein model-input workflow. The config accepts the FASTA text, tokenizer profile, and model-input policy. The output includes parsed records, tokenized records, model-ready inputs, readiness issues, and provenance metadata.

```javascript
const output = runWorkflow({
  fastaText: '>seq1\nACDEFGHIKLMNPQRSTVWY\n',
  profile: 'protein-20-special',
  maxLength: 512,
});

console.log(output.modelInput.records[0].input_ids);
console.log(output.provenance.inputHash);
console.log(output.readinessIssues);
```

---

### `package`

Validate and inspect bio-rs package manifests in the browser.

#### `validateManifest(manifestJson: string): ValidationReport`

Validate a package manifest JSON string against the bio-rs package schema. Returns a structured report with typed issue codes, severity levels, and paths into the manifest.

```javascript
const report = validateManifest(manifestJson);
console.log(report.valid);
console.log(report.issues);
```

#### `planRuntimeBridge(manifestJson: string): RuntimeBridgeReport`

Plan a runtime bridge from a package manifest. The report describes backend compatibility, model artifact metadata, runtime target alignment, and any capability gaps between the manifest's declared runtime and the current environment.

```javascript
const bridge = planRuntimeBridge(manifestJson);
console.log(bridge.compatible);
console.log(bridge.backendCapabilities);
```

---

## TypeScript Interfaces

These interfaces describe the serde payloads returned by the WASM functions. They are maintained alongside the JSON schemas in `schemas/` and the Rust struct definitions in `biors-core`.

### `ProteinSequence`

```typescript
interface ProteinSequence {
  id: string;
  description: string | null;
  sequence: string;
  kind: 'protein' | 'dna' | 'rna' | 'unknown';
}
```

### `ValidationReport`

```typescript
interface ValidationIssue {
  code: string;
  severity: 'error' | 'warning' | 'info';
  message: string;
  path: string | null;
  line: number | null;
}

interface ValidationReport {
  valid: boolean;
  records: ValidationRecord[];
  issues: ValidationIssue[];
}

interface ValidationRecord {
  id: string;
  valid: boolean;
  kind: string;
  issues: ValidationIssue[];
}
```

### `TokenizedRecord`

```typescript
interface TokenizedRecord {
  id: string;
  tokens: number[];
  tokenCount: number;
  profile: string;
}
```

### `ModelInput`

```typescript
interface ModelInputRecord {
  id: string;
  input_ids: number[];
  attention_mask: number[];
  truncated: boolean;
  originalLength: number;
}

interface ModelInput {
  records: ModelInputRecord[];
  maxLength: number;
  paddingTokenId: number;
}
```

### `WorkflowConfig`

```typescript
interface WorkflowConfig {
  fastaText: string;
  profile: string;
  maxLength: number;
}
```

### `WorkflowOutput`

```typescript
interface WorkflowProvenance {
  inputHash: string;
  profile: string;
  timestamp: string;
  version: string;
}

interface WorkflowReadinessIssue {
  code: string;
  severity: 'error' | 'warning';
  message: string;
}

interface WorkflowOutput {
  records: ProteinSequence[];
  tokenized: TokenizedRecord[];
  modelInput: ModelInput;
  provenance: WorkflowProvenance;
  readinessIssues: WorkflowReadinessIssue[];
}
```

### `RuntimeBridgeReport`

```typescript
interface BackendCapabilitiesSummary {
  backend: string;
  target: string;
  supportsFp16: boolean;
  supportsWebGpu: boolean;
}

interface RuntimeBridgeReport {
  compatible: boolean;
  backendCapabilities: BackendCapabilitiesSummary;
  modelArtifact: ModelArtifactMetadata | null;
  issues: ValidationIssue[];
}

interface ModelArtifactMetadata {
  name: string;
  version: string;
  architecture: string;
  task: string;
  source: string;
}
```

---

## Browser Package Manifest Workflow

Because the WASM build has no filesystem access, the browser-side workflow for package validation is slightly different from the CLI workflow:

1. **JavaScript loads the files.** Use `fetch`, `FileReader`, or drag-and-drop to read `manifest.json` and any artifacts into memory as strings or `Uint8Array` buffers.
2. **WASM validates the manifest.** Pass the manifest JSON string to `validateManifest`. The WASM side checks schema compliance, required fields, checksum formats, and cross-reference consistency.
3. **JavaScript displays the report.** Render `ValidationReport.issues` in the UI. Because artifact contents are already in memory, you can verify SHA-256 hashes with standard Web Crypto APIs if needed.

```javascript
async function validatePackageInBrowser(manifestFile, artifactFiles) {
  await init();

  // 1. JS loads files
  const manifestText = await manifestFile.text();

  // 2. WASM validates manifest structure
  const report = validateManifest(manifestText);

  // 3. JS verifies artifact hashes with Web Crypto
  for (const file of artifactFiles) {
    const buffer = await file.arrayBuffer();
    const hash = await crypto.subtle.digest('SHA-256', buffer);
    // compare against manifest checksums
  }

  return report;
}
```

This split keeps the Rust side focused on schema and policy logic while the browser handles I/O.

---

## Limitations

The WASM subset is intentionally smaller than the native Rust API. These capabilities are **not available** in the browser build:

| Capability | Status in WASM | Reason |
|---|---|---|
| Filesystem access | Not available | Browsers have no `std::fs`. All input must be passed as in-memory strings or `Uint8Array`. |
| External process spawning | Not available | Browsers cannot run `std::process::Command`. The `runtime` module is hidden. |
| Network I/O | Not available | No `std::net`. Use `fetch` in JavaScript, then pass the result to WASM. |
| Thread spawning | Not available | No `std::thread`. All computation is single-threaded inside the WASM module. |
| Package artifact reads | Not available | `read_package_file` and `validate_package_manifest_artifacts` are hidden. JS must load bytes. |
| Reader-based FASTA APIs | Not available | `parse_fasta_records_reader` and `validate_fasta_reader` require `BufRead`. Use string variants. |

If you need any of these, use the `biors` CLI, the native Rust crate, or the planned Python bindings instead.

---

## Performance Notes

- **String vs. bytes:** For FASTA text under a few megabytes, passing a JavaScript `string` is fine. For very large FASTA files, consider loading the file as a `Uint8Array`, decoding it to a string in a worker, and then passing the string to WASM. The boundary cost is dominated by UTF-8 validation and copy, not by the Rust parse itself.
- **Zero copy is not guaranteed:** `wasm-bindgen` copies data across the JS/WASM boundary. Large `ModelInput` batches will allocate on both sides.
- **Tokenization is fast:** The protein-20 tokenizer is a small lookup table. In benchmarks, `biors-core` tokenizes at over 200 MB/s on native targets. Browser performance depends on the JS engine and WASM runtime but should be competitive with pure-JS tokenizers for typical inputs.
- **Avoid repeated `init` calls:** `init` is idempotent in most glue layers, but calling it more than once wastes time. Initialize once at application startup.

---

## Complete Example

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>bio-rs WASM Demo</title>
</head>
<body>
  <input type="file" id="fastaFile" accept=".fasta,.fa,.txt">
  <pre id="output"></pre>

  <script type="module">
    import init, {
      parseFasta,
      validateFasta,
      tokenize,
      buildModelInput,
      runWorkflow,
    } from '@bio-rs/core-wasm';

    async function main() {
      await init();

      document.getElementById('fastaFile').addEventListener('change', async (e) => {
        const file = e.target.files[0];
        if (!file) return;

        const text = await file.text();

        // Validate
        const validation = validateFasta(text, 'auto');
        if (!validation.valid) {
          document.getElementById('output').textContent =
            'Validation failed:\n' + JSON.stringify(validation.issues, null, 2);
          return;
        }

        // Parse and tokenize
        const records = parseFasta(text);
        const tokenized = tokenize(records, 'protein-20-special');
        const modelInput = buildModelInput(tokenized, 512);

        // Or use the workflow shortcut
        const workflow = runWorkflow({
          fastaText: text,
          profile: 'protein-20-special',
          maxLength: 512,
        });

        document.getElementById('output').textContent = JSON.stringify({
          recordCount: records.length,
          firstTokens: tokenized[0]?.tokens,
          firstInputIds: modelInput.records[0]?.input_ids,
          provenance: workflow.provenance,
        }, null, 2);
      });
    }

    main();
  </script>
</body>
</html>
```

---

## Design Reference

The WASM API boundary was designed in `docs/superpowers/specs/2026-05-22-phase7-external-interface-0.43-design.md`. That document contains the full module-by-module safety audit, the compatibility matrix across Rust native, WASM, and Python targets, and the implementation roadmap.

For questions about the native Rust API, see `docs/rust-api.md` (planned). For Python interop, see `docs/python-interop.md` and `docs/python-api.md` (planned).

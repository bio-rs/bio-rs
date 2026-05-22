# biors-wasm v0.45.0 Design Document

## 1. Overview

This document designs the `biors-wasm` crate: a WebAssembly/JavaScript API layer for `bio-rs` that exposes `biors-core` functionality to browser and Node.js consumers. The crate is optional, does not affect default builds, and produces an npm-publishable package `@bio-rs/biors-wasm`.

**Design principles:**
- Browser-safe: no filesystem access, no process spawning, no external dependencies
- JSON-compatible: all inputs/outputs are JSON-serializable objects or byte arrays
- Thin wrapper: delegate all domain logic to `biors-core`; `biors-wasm` is purely a bindgen boundary
- Optional: gated behind workspace membership and crate features; never compiled by default
- TypeScript-first: generated `.d.ts` plus custom TypeScript interfaces for consumer ergonomics

---

## 2. Research Summary

### 2.1 wasm-bindgen / wasm-pack Best Practices (2024–2025)

- **wasm-bindgen 0.2.118** (latest stable, April 2026) is the baseline. MSRV is Rust 1.77.
- `wasm-pack` generates `pkg/` with `.wasm`, `.js` glue, `.d.ts`, `package.json`, and `README`.
- `--target web` produces ES modules for direct browser use (manual `init()` required).
- `--target bundler` produces code for Webpack/Rollup/Vite with `sideEffects: false`.
- `wasm-pack` auto-generates TypeScript declarations by default (`--typescript` is on by default).
- `typescript_custom_section` attribute allows appending custom TypeScript types to generated `.d.ts`.
- `typescript_type` attribute on `extern "C"` types lets Rust functions accept/return custom TS interfaces.
- **Key pitfall**: `Vec<T>` where `T` is a wasm-bindgen-exported struct consumes ownership on the JS side. Pass by `&[T]` or use `JsValue` + manual deserialization for better ergonomics.
- **Key pitfall**: `&[u8]` maps to `Uint8Array` (good), but `ArrayBuffer` must be wrapped in `Uint8Array` before passing.
- **Recommendation from community**: Prefix exported Rust types with `Wasm*` and set `js_name` to the unprefixed name to avoid name collisions.

### 2.2 TypeScript Definitions Strategy

Three complementary approaches:
1. **Auto-generated**: `wasm-pack` produces `biors_wasm.d.ts` from `#[wasm_bindgen]` exports. This covers functions and classes.
2. **Custom sections**: `#[wasm_bindgen(typescript_custom_section)]` const strings append custom interfaces (e.g., `FastaRecord`, `ValidationReport`) to the generated `.d.ts`.
3. **Hand-written `index.d.ts`**: A top-level declaration file that re-exports and refines the auto-generated types for the npm package consumer. This is the public contract.

No external tool like `wasm-bindgen-typescript` is needed; wasm-bindgen natively supports this since 0.2.90+.

### 2.3 Optional Features in WASM Builds

- `biors-core` already compiles for `wasm32-unknown-unknown`.
- Tokenization profiles (`protein-20`, `protein-20-special`) are runtime-selected via `ProteinTokenizerConfig`.
- No additional Cargo features are required in `biors-wasm` for profile selection; profiles are passed as string arguments.
- If `biors-core` later adds heavy optional dependencies (e.g., Candle backend), `biors-wasm` will depend on `biors-core` with `default-features = false` and selectively enable only WASM-safe features.

---

## 3. Crate Structure

```
packages/rust/biors-wasm/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs              # wasm_bindgen entry point, module exports
│   ├── fasta.rs            # parseFasta, validateFasta wrappers
│   ├── tokenize.rs         # tokenize wrapper
│   ├── model_input.rs      # buildModelInput wrapper
│   ├── workflow.rs         # runWorkflow wrapper
│   └── types.rs            # wasm_bindgen structs + typescript_custom_section
├── pkg/                    # wasm-pack output (gitignored)
├── package.json            # npm package manifest (template for wasm-pack)
├── index.d.ts              # hand-written public TypeScript declarations
└── .gitignore
```

### 3.1 Workspace Integration

**`Cargo.toml` (workspace root — NOT to be edited by this design task):**

```toml
[workspace]
members = [
    "packages/rust/biors",
    "packages/rust/biors-backend-candle",
    "packages/rust/biors-core",
    # "packages/rust/biors-wasm",  # commented out — optional, not default
]
```

> **Rationale**: The crate must be optional and not affect default builds. CI will explicitly build it via `cargo check -p biors-wasm --target wasm32-unknown-unknown`. When stabilized, it can be uncommented.

**`packages/rust/biors-wasm/Cargo.toml`:**

```toml
[package]
name = "biors-wasm"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "WebAssembly/JavaScript bindings for bio-rs core biological sequence processing."
readme = "../../../README.md"
keywords = ["bioinformatics", "wasm", "javascript", "tokenizer", "ai"]
categories = ["science", "wasm"]

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
biors-core = { workspace = true, default-features = false }
wasm-bindgen = "0.2.118"
serde = { workspace = true }
serde_json = { workspace = true }
js-sys = "0.3.70"

# Optional: enable console_error_panic_hook for better debugging in browsers
console_error_panic_hook = { version = "0.1.7", optional = true }

[features]
default = ["console_error_panic_hook"]
```

> **Notes**:
> - `cdylib` is required for wasm-pack to produce the `.wasm` binary.
> - `rlib` allows local `cargo check` without a WASM target.
> - `biors-core` uses `default-features = false` to prevent pulling in any future non-WASM-safe features.
> - `js-sys` provides `Uint8Array`, `Array`, and other JS primitives for zero-copy where possible.

---

## 4. wasm-bindgen API Surface Design

### 4.1 Design Patterns

- **Input**: `Uint8Array` for raw bytes, `String` for text parameters, `JsValue` for JSON objects.
- **Output**: `JsValue` containing a `serde_json::Value` serialized to a JS object. This avoids defining dozens of wasm-bindgen structs while preserving full type information.
- **Error handling**: Rust `Result<T, E>` maps to JS exceptions via `wasm-bindgen`. For structured errors, we return a JSON object with `{ success: false, error: { code, message } }`.
- **No file I/O**: All functions accept in-memory data only.

### 4.2 Exported Functions

```rust
// src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Parse FASTA bytes into an array of sequence records.
///
/// Input: Uint8Array containing ASCII FASTA text.
/// Output: Array of { id: string, sequence: string }.
#[wasm_bindgen(js_name = parseFasta)]
pub fn parse_fasta(bytes: &[u8]) -> Result<JsValue, JsValue>;

/// Validate FASTA bytes and return a structured validation report.
///
/// Input: Uint8Array containing ASCII FASTA text; kind is "auto", "protein", "dna", or "rna".
/// Output: { valid: boolean, records: number, validRecords: number, warningCount: number, errorCount: number, sequences: Array<ValidatedSequence> }.
#[wasm_bindgen(js_name = validateFasta)]
pub fn validate_fasta(bytes: &[u8], kind: String) -> Result<JsValue, JsValue>;

/// Tokenize parsed FASTA records using a named profile.
///
/// Input: Array of { id: string, sequence: string } (as returned by parseFasta);
///        profile is "protein-20" or "protein-20-special".
/// Output: { inputIds: number[][], attentionMask: number[][], ids: string[], records: Array<TokenizedRecord> }.
#[wasm_bindgen(js_name = tokenize)]
pub fn tokenize(records: JsValue, profile: String) -> Result<JsValue, JsValue>;

/// Build model-ready input from tokenized records.
///
/// Input: Array of TokenizedRecord (as returned by tokenize.records);
///        maxLength is the per-record token limit.
/// Output: { records: Array<ModelInputRecord> }.
#[wasm_bindgen(js_name = buildModelInput)]
pub fn build_model_input(tokenized: JsValue, max_length: usize) -> Result<JsValue, JsValue>;

/// Run the full validation -> tokenization -> model-input workflow.
///
/// Input: Config object (see TypeScript interfaces below).
/// Output: SequenceWorkflowOutput object.
#[wasm_bindgen(js_name = runWorkflow)]
pub fn run_workflow(config: JsValue) -> Result<JsValue, JsValue>;
```

### 4.3 Internal Module Breakdown

| Module | Responsibility |
|--------|---------------|
| `src/lib.rs` | `wasm_bindgen` macro invocations, `start` hook, re-exports |
| `src/types.rs` | `typescript_custom_section` consts, internal helper structs for JSON serialization |
| `src/fasta.rs` | Wraps `biors_core::fasta::parse_fasta_records` and `validate_fasta_input_with_kind` |
| `src/tokenize.rs` | Wraps `biors_core::tokenizer::tokenize_protein` and config loading |
| `src/model_input.rs` | Wraps `biors_core::model_input::build_model_inputs_checked` |
| `src/workflow.rs` | Wraps `biors_core::workflow::prepare_protein_model_input_workflow` |

---

## 5. TypeScript Definitions Strategy

### 5.1 Auto-Generated Types

`wasm-pack build` produces `pkg/biors_wasm.d.ts` containing:
- `export function parseFasta(bytes: Uint8Array): any;`
- `export function validateFasta(bytes: Uint8Array, kind: string): any;`
- etc.

These use `any` for return types because we return `JsValue`. We refine these in the hand-written layer.

### 5.2 Custom TypeScript Sections (in Rust)

In `src/types.rs`:

```rust
#[wasm_bindgen(typescript_custom_section)]
const TS_FASTR_RECORD: &'static str = r#"
export interface FastaRecord {
    id: string;
    sequence: string;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_VALIDATION_REPORT: &'static str = r#"
export interface ValidationIssue {
    residue: string;
    position: number;
}

export interface ValidatedSequence {
    id: string;
    sequence: string;
    valid: boolean;
    warnings: ValidationIssue[];
    errors: ValidationIssue[];
}

export interface ValidationReport {
    valid: boolean;
    records: number;
    validRecords: number;
    warningCount: number;
    errorCount: number;
    sequences: ValidatedSequence[];
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_TOKENIZED: &'static str = r#"
export interface TokenizedRecord {
    id: string;
    tokens: number[];
    length: number;
    alphabet: string;
    valid: boolean;
    warnings: ValidationIssue[];
    errors: ValidationIssue[];
}

export interface TokenizeOutput {
    inputIds: number[][];
    attentionMask: number[][];
    ids: string[];
    records: TokenizedRecord[];
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_MODEL_INPUT: &'static str = r#"
export interface ModelInputRecord {
    id: string;
    inputIds: number[];
    attentionMask: number[];
    truncated: boolean;
}

export interface ModelInputOutput {
    records: ModelInputRecord[];
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_WORKFLOW: &'static str = r#"
export interface WorkflowConfig {
    fastaBytes: Uint8Array;
    kind?: "auto" | "protein" | "dna" | "rna";
    profile?: "protein-20" | "protein-20-special";
    maxLength: number;
    padding?: "fixed_length" | "no_padding";
    padTokenId?: number;
}

export interface WorkflowReadinessIssue {
    code: string;
    id: string;
    warningCount: number;
    errorCount: number;
    message: string;
}

export interface WorkflowOutput {
    workflow: string;
    modelReady: boolean;
    validation: ValidationReport;
    tokenization: {
        summary: {
            records: number;
            totalLength: number;
            validRecords: number;
            warningCount: number;
            errorCount: number;
        };
        records: TokenizedRecord[];
    };
    modelInput: ModelInputOutput | null;
    readinessIssues: WorkflowReadinessIssue[];
    provenance: {
        biorsCoreVersion: string;
        inputHash: string;
        normalization: string;
        validationAlphabet: string;
        tokenizer: {
            name: string;
            vocabSize: number;
            unknownTokenId: number;
        };
        modelInputPolicy: {
            maxLength: number;
            padTokenId: number;
            padding: string;
        };
    };
}
"#;
```

### 5.3 Hand-Written `index.d.ts`

This is the public npm package contract. It imports from the generated module and re-exports with refined types:

```typescript
// index.d.ts
export {
    parseFasta,
    validateFasta,
    tokenize,
    buildModelInput,
    runWorkflow,
    default as init,
} from "./biors_wasm.js";

export type {
    FastaRecord,
    ValidationIssue,
    ValidatedSequence,
    ValidationReport,
    TokenizedRecord,
    TokenizeOutput,
    ModelInputRecord,
    ModelInputOutput,
    WorkflowConfig,
    WorkflowReadinessIssue,
    WorkflowOutput,
} from "./biors_wasm.d.ts";

// Refine function signatures for consumers
declare module "./biors_wasm.js" {
    export function parseFasta(bytes: Uint8Array): FastaRecord[];
    export function validateFasta(bytes: Uint8Array, kind: string): ValidationReport;
    export function tokenize(records: FastaRecord[], profile: string): TokenizeOutput;
    export function buildModelInput(tokenized: TokenizedRecord[], maxLength: number): ModelInputOutput;
    export function runWorkflow(config: WorkflowConfig): WorkflowOutput;
}
```

---

## 6. npm Package Structure

### 6.1 Package Name

`@bio-rs/biors-wasm`

Scoped to the `bio-rs` npm organization. This prevents name squatting and aligns with the Rust workspace naming.

### 6.2 `package.json` Template

`wasm-pack` auto-generates `pkg/package.json`, but we provide a template in the crate root:

```json
{
  "name": "@bio-rs/biors-wasm",
  "version": "0.45.0",
  "description": "WebAssembly bindings for bio-rs biological sequence processing",
  "license": "MIT OR Apache-2.0",
  "repository": {
    "type": "git",
    "url": "https://github.com/bio-rs/bio-rs.git",
    "directory": "packages/rust/biors-wasm"
  },
  "files": [
    "biors_wasm_bg.wasm",
    "biors_wasm.js",
    "biors_wasm.d.ts",
    "index.d.ts",
    "README.md"
  ],
  "module": "biors_wasm.js",
  "types": "index.d.ts",
  "sideEffects": false,
  "keywords": [
    "bioinformatics",
    "protein",
    "tokenizer",
    "wasm",
    "rust"
  ]
}
```

> **Note**: `wasm-pack` will overwrite some fields during build. The template ensures `sideEffects: false` and correct `types` entry point are preserved.

### 6.3 Multi-Target Distribution

For v0.45.0, we publish a single `--target bundler` build as the default npm artifact. Consumers using bundlers (Vite, Webpack, Rollup, esbuild) get the best experience.

A `--target web` build is produced in CI and attached to GitHub Releases as a separate tarball for direct-browser users.

Future versions may adopt a multi-target package structure (e.g., `pkg-web/`, `pkg-bundler/`, `pkg-nodejs/`) if demand justifies the complexity.

---

## 7. Browser-Safe Wrapper Design

### 7.1 Constraints

| Constraint | Implementation |
|-----------|----------------|
| No filesystem access | Never use `std::fs` or `std::path`. All inputs are `&[u8]` or `String`. |
| No process spawning | Never use `std::process::Command`. |
| No external network | No `reqwest`, no `curl`. All data passed by caller. |
| No platform-specific code | `biors-core` already compiles for `wasm32-unknown-unknown`. |
| Deterministic | Same input bytes → same output. No randomness, no time dependence. |

### 7.2 Byte Array Handling

- `parseFasta(bytes: &[u8])` → converts bytes to `&str` via `std::str::from_utf8` (FASTA is ASCII/UTF-8 text).
- `validateFasta(bytes: &[u8], kind: String)` → same path, then dispatches to `biors-core::sequence::kind_validation`.
- On the JS side, consumers pass `Uint8Array`. `wasm-bindgen` automatically copies the array into WASM linear memory.

### 7.3 JSON Serialization Boundary

All complex return types go through `serde_json`:

```rust
fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_json::to_string(value)
        .map_err(|e| JsValue::from_str(&e.to_string()))
        .map(|s| JsValue::from_str(&s))
        .map(|js_str| JSON::parse(&js_str).unwrap()) // js_sys::JSON::parse
}
```

This is slightly less efficient than direct `wasm-bindgen` struct mapping, but it:
- Avoids exporting dozens of internal `biors-core` types through the boundary
- Keeps the JS API stable even when `biors-core` internal types change
- Allows consumers to receive plain JS objects, not wasm-bindgen class instances

### 7.4 Error Mapping

Rust errors are mapped to JS `Error` objects:

```rust
fn map_err<E: std::fmt::Display>(e: E) -> JsValue {
    JsValue::from_str(&e.to_string())
}
```

`wasm-bindgen` automatically turns `Result<T, JsValue>` into either a return value or a thrown exception.

---

## 8. Implementation Plan

### Phase 1: Crate scaffolding (1 day)
1. Create `packages/rust/biors-wasm/` directory structure.
2. Write `Cargo.toml` with `wasm-bindgen`, `js-sys`, `serde`, `serde_json`, `biors-core` dependencies.
3. Add `cdylib` crate type.
4. Write minimal `src/lib.rs` with `wasm_bindgen(start)` and `console_error_panic_hook`.
5. Verify `cargo check --target wasm32-unknown-unknown` passes.

### Phase 2: Core wrappers (2 days)
1. Implement `src/fasta.rs`: `parse_fasta`, `validate_fasta`.
2. Implement `src/tokenize.rs`: `tokenize` with profile string dispatch.
3. Implement `src/model_input.rs`: `build_model_input`.
4. Implement `src/workflow.rs`: `run_workflow` with config deserialization.
5. Implement `src/types.rs`: `typescript_custom_section` blocks.
6. Add unit tests for each wrapper using `wasm-bindgen-test`.

### Phase 3: TypeScript & npm packaging (1 day)
1. Write `index.d.ts` with full public interface.
2. Create `package.json` template.
3. Run `wasm-pack build --target bundler` and verify generated `pkg/`.
4. Verify TypeScript consumers can import and type-check against `index.d.ts`.

### Phase 4: Browser example (1 day)
1. Create `examples/wasm-browser/` with a minimal standalone HTML file.
2. Include `pkg/` artifacts (or load from relative path in dev mode).
3. Demonstrate: parse → validate → tokenize → buildModelInput → runWorkflow.
4. Serve via `python3 -m http.server` or `npx serve`.

### Phase 5: CI integration (1 day)
1. Add `wasm-pack` build step to GitHub Actions.
2. Build `--target web` and `--target bundler`.
3. Run `wasm-bindgen-test` (headless browser tests).
4. Attach `pkg/` tarball to releases.
5. Publish to npm on version tags (manual trigger or automated with `NPM_TOKEN`).

---

## 9. File Structure (Target State)

```
bio-rs/
├── packages/rust/
│   ├── biors/
│   ├── biors-backend-candle/
│   ├── biors-core/
│   └── biors-wasm/
│       ├── Cargo.toml
│       ├── README.md
│       ├── .gitignore
│       ├── package.json          # npm template
│       ├── index.d.ts            # public TS declarations
│       ├── src/
│       │   ├── lib.rs
│       │   ├── types.rs
│       │   ├── fasta.rs
│       │   ├── tokenize.rs
│       │   ├── model_input.rs
│       │   └── workflow.rs
│       └── tests/
│           └── web.rs            # wasm-bindgen-test suite
├── examples/
│   └── wasm-browser/
│       ├── index.html
│       ├── main.js
│       └── README.md
├── .github/
│   └── workflows/
│       └── wasm.yml              # CI for WASM builds & npm publish
└── scripts/
    └── check.sh                  # already includes wasm32 check
```

---

## 10. CI / Build Strategy

### 10.1 GitHub Actions Workflow (`.github/workflows/wasm.yml`)

```yaml
name: WASM

on:
  push:
    branches: [main]
    tags: ["v*"]
  pull_request:
    paths:
      - "packages/rust/biors-core/**"
      - "packages/rust/biors-wasm/**"
      - ".github/workflows/wasm.yml"

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: jetli/wasm-pack-action@v0.4.0
      - run: cargo check -p biors-wasm --target wasm32-unknown-unknown
      - run: wasm-pack build packages/rust/biors-wasm --target bundler --out-dir pkg-bundler
      - run: wasm-pack build packages/rust/biors-wasm --target web --out-dir pkg-web
      - name: Upload bundler artifact
        uses: actions/upload-artifact@v4
        with:
          name: biors-wasm-bundler
          path: packages/rust/biors-wasm/pkg-bundler/
      - name: Upload web artifact
        uses: actions/upload-artifact@v4
        with:
          name: biors-wasm-web
          path: packages/rust/biors-wasm/pkg-web/

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: jetli/wasm-pack-action@v0.4.0
      - run: wasm-pack test --headless --chrome packages/rust/biors-wasm
      - run: wasm-pack test --headless --firefox packages/rust/biors-wasm

  publish:
    if: startsWith(github.ref, 'refs/tags/v')
    needs: [check, test]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: jetli/wasm-pack-action@v0.4.0
      - uses: actions/setup-node@v4
        with:
          registry-url: "https://registry.npmjs.org"
      - run: wasm-pack build packages/rust/biors-wasm --target bundler --scope bio-rs
      - run: |
          cd packages/rust/biors-wasm/pkg
          npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

### 10.2 Local Build Commands

```bash
# Check compilation (no WASM target needed if using rlib)
cargo check -p biors-wasm

# Check for WASM target
cargo check -p biors-wasm --target wasm32-unknown-unknown

# Build for bundlers (npm package)
wasm-pack build packages/rust/biors-wasm --target bundler

# Build for direct browser use
wasm-pack build packages/rust/biors-wasm --target web --out-dir pkg-web

# Run browser tests
wasm-pack test --headless --chrome packages/rust/biors-wasm
```

### 10.3 Integration with Existing `scripts/check.sh`

The existing `scripts/check.sh` already runs:
```bash
cargo check --target wasm32-unknown-unknown -p biors-core
```

When `biors-wasm` is added to the workspace (or built explicitly), add:
```bash
cargo check --target wasm32-unknown-unknown -p biors-wasm
wasm-pack build packages/rust/biors-wasm --target bundler --mode no-install
```

---

## 11. Example: `examples/wasm-browser/`

A minimal standalone HTML demo. Not a product. No build step required for the example itself.

```html
<!-- examples/wasm-browser/index.html -->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>bio-rs WASM Demo</title>
</head>
<body>
  <h1>bio-rs WASM Demo</h1>
  <pre id="output">Loading WASM...</pre>

  <script type="module">
    import init, { parseFasta, validateFasta, tokenize, buildModelInput, runWorkflow } from "../../packages/rust/biors-wasm/pkg/biors_wasm.js";

    async function main() {
      await init();

      const fastaText = ">seq1\nACDE\n>seq2\nFGHI\n";
      const bytes = new TextEncoder().encode(fastaText);

      const parsed = parseFasta(bytes);
      console.log("Parsed:", parsed);

      const validated = validateFasta(bytes, "protein");
      console.log("Validated:", validated);

      const tokenized = tokenize(parsed, "protein-20");
      console.log("Tokenized:", tokenized);

      const modelInput = buildModelInput(tokenized.records, 8);
      console.log("Model Input:", modelInput);

      const workflow = runWorkflow({
        fastaBytes: bytes,
        kind: "protein",
        profile: "protein-20",
        maxLength: 8,
        padding: "fixed_length",
        padTokenId: 0,
      });
      console.log("Workflow:", workflow);

      document.getElementById("output").textContent = JSON.stringify(workflow, null, 2);
    }

    main().catch(console.error);
  </script>
</body>
</html>
```

**Serving the example:**
```bash
cd examples/wasm-browser
python3 -m http.server 8080
# open http://localhost:8080
```

> **Note**: The relative import path `../../packages/rust/biors-wasm/pkg/biors_wasm.js` assumes the example is served from the repo root. For a standalone distribution, the example would load from a CDN or a copied `pkg/` directory.

---

## 12. Optional Features & Future Extensibility

### 12.1 Tokenization Profiles

Profiles are runtime-selected strings, not compile-time features:
- `"protein-20"` → `ProteinTokenizerProfile::Protein20`
- `"protein-20-special"` → `ProteinTokenizerProfile::Protein20Special`

Future profiles (e.g., `"protein-25"`, `"dna-4"`) are added in `biors-core` and automatically available in `biors-wasm` without recompilation.

### 12.2 Feature-Gated Heavier Logic

If `biors-core` later adds non-WASM-safe features (e.g., Candle backend, async I/O), `biors-wasm` will:
- Depend on `biors-core` with `default-features = false`
- Explicitly enable only WASM-safe features in `[dependencies]`
- Use `cfg(target_arch = "wasm32")` to stub out unsupported APIs with clear error messages

### 12.3 Future npm Package Variants

| Variant | Target | Use Case |
|---------|--------|----------|
| `@bio-rs/biors-wasm` | `bundler` | Default for Vite/Webpack/Rollup |
| `@bio-rs/biors-wasm/web` | `web` | Direct ES module import in browser |
| `@bio-rs/biors-wasm/nodejs` | `nodejs` | Node.js CommonJS |

For v0.45.0, only the bundler variant is published to npm. Web and Node.js variants are build artifacts attached to GitHub Releases.

---

## 13. Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| `biors-core` adds `std::fs` dependency | `biors-wasm` uses `default-features = false`; CI catches breakage via `wasm32` build |
| Large `.wasm` binary size | Enable `wasm-opt` (default in wasm-pack); profile with `twiggy`; consider `wee_alloc` |
| JSON serialization overhead | Acceptable for v0.45.0 (data sizes are small: FASTA records, token vectors). Future: direct `wasm-bindgen` struct mapping |
| TypeScript types drift from Rust | `typescript_custom_section` lives next to wrapper code; CI runs `tsc --noEmit` on `examples/wasm-browser/` |
| npm publish token exposure | Use GitHub Actions `secrets.NPM_TOKEN` with restricted npm automation token |

---

## 14. Summary

This design produces:
- A new `biors-wasm` crate at `packages/rust/biors-wasm/`
- A `wasm-bindgen` API exposing `parseFasta`, `validateFasta`, `tokenize`, `buildModelInput`, and `runWorkflow`
- Auto-generated + custom TypeScript definitions
- An npm package `@bio-rs/biors-wasm` for bundler consumers
- A minimal browser demo at `examples/wasm-browser/`
- CI automation for build, test, and publish

The crate is optional, does not affect default builds, and preserves the existing `biors-core` WASM32 compatibility guarantee.

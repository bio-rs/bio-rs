# biors-wasm

WebAssembly/JavaScript bindings for [bio-rs](https://github.com/bio-rs/bio-rs) core biological sequence processing.

## Usage

```javascript
import {
  browserExecutionPolicy,
  inspectBrowserFile,
  validateBrowserFile,
  parseFasta,
  validateFasta,
  tokenize,
  tokenizeBrowserFile,
  buildModelInputWithPolicy,
  runWorkflow,
} from "@bio-rs/biors-wasm";

const fastaText = ">seq1\nACDE\n>seq2\nFGHI\n";
const bytes = new TextEncoder().encode(fastaText);

const parsed = parseFasta(bytes);
const validated = validateFasta(bytes, "protein");
const tokenized = tokenize(parsed, "protein-20");
const dnaTokens = tokenize(parseFasta(new TextEncoder().encode(">dna\nACGT\n")), "dna-iupac");
const modelInput = buildModelInputWithPolicy(tokenized.records, 8, 0, "fixed_length");
const workflow = runWorkflow({
  fastaBytes: bytes,
  maxLength: 8,
  padding: "fixed_length",
  padTokenId: 0,
});
const dnaWorkflow = runWorkflow({
  fastaBytes: new TextEncoder().encode(">dna\nACGT\n"),
  kind: "dna",
  profile: "dna-iupac",
  maxLength: 8,
});

const policy = browserExecutionPolicy();
const fileInput = {
  name: "protein.fasta",
  format: "fasta",
  bytes,
  kind: "protein",
  profile: "protein-20",
};
const inspected = inspectBrowserFile(fileInput);
const browserValidation = validateBrowserFile(fileInput);
const browserTokens = tokenizeBrowserFile(fileInput);
```

The browser helpers are local-first by contract: they accept caller-owned
`Uint8Array` input, enforce the documented size policy, do not call `fetch`, do
not upload input data, and do not persist records.

## Development

```bash
# Check compilation
cargo check -p biors-wasm --target wasm32-unknown-unknown

# Build for bundlers
wasm-pack build crates/biors-wasm --target bundler

# Enable browser panic stack traces for local debugging
wasm-pack build crates/biors-wasm --target bundler --features console_error_panic_hook

# Run browser tests
wasm-pack test --headless --chrome crates/biors-wasm
```

## License

MIT OR Apache-2.0

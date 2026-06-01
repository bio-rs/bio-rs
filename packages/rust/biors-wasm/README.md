# biors-wasm

WebAssembly/JavaScript bindings for [bio-rs](https://github.com/bio-rs/bio-rs) core biological sequence processing.

## Usage

```javascript
import {
  parseFasta,
  validateFasta,
  tokenize,
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
```

## Development

```bash
# Check compilation
cargo check -p biors-wasm --target wasm32-unknown-unknown

# Build for bundlers
wasm-pack build packages/rust/biors-wasm --target bundler

# Enable browser panic stack traces for local debugging
wasm-pack build packages/rust/biors-wasm --target bundler --features console_error_panic_hook

# Run browser tests
wasm-pack test --headless --chrome packages/rust/biors-wasm
```

## License

MIT OR Apache-2.0

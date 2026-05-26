# biors-wasm

WebAssembly/JavaScript bindings for [bio-rs](https://github.com/bio-rs/bio-rs) core biological sequence processing.

## Usage

```javascript
import init, {
  parseFasta,
  validateFasta,
  tokenize,
  buildModelInputWithPolicy,
  runWorkflow,
} from "@bio-rs/biors-wasm";

await init();

const fastaText = ">seq1\nACDE\n>seq2\nFGHI\n";
const bytes = new TextEncoder().encode(fastaText);

const parsed = parseFasta(bytes);
const validated = validateFasta(bytes, "protein");
const tokenized = tokenize(parsed, "protein-20");
const modelInput = buildModelInputWithPolicy(tokenized.records, 8, 0, "fixed_length");
const workflow = runWorkflow({
  fastaBytes: bytes,
  maxLength: 8,
  padding: "fixed_length",
  padTokenId: 0,
});
```

## Development

```bash
# Check compilation
cargo check -p biors-wasm --target wasm32-unknown-unknown

# Build for bundlers
wasm-pack build packages/rust/biors-wasm --target bundler

# Run browser tests
wasm-pack test --headless --chrome packages/rust/biors-wasm
```

## License

MIT OR Apache-2.0

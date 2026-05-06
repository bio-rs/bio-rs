# Launch Demo

This demo is the short path for showing bio-rs to researchers, contributors,
and technical evaluators without inventing a separate sample app.

## Dataset

The committed dataset is `examples/launch-demo.fasta`.

It contains short protein fragments from familiar human research targets:

- BRCA1
- CFTR
- TP53

The dataset is intentionally small enough to inspect in a terminal and large
enough to exercise multi-record validation, tokenization, and model-input
generation.

## Run From A Checkout

```bash
sh scripts/launch-demo.sh --cargo
```

## Run With An Installed Binary

```bash
cargo install biors --version 0.15.2
biors --version
sh scripts/launch-demo.sh
```

## Website Demo Script

1. Show the first README screen: bio-rs converts FASTA into validated,
   model-ready JSON for bio-AI workflows.
2. Open `examples/launch-demo.fasta` and point out that the input is ordinary
   multi-record protein FASTA.
3. Run `biors doctor` to show platform, toolchain, WASM-target, and fixture
   readiness.
4. Run `biors seq validate examples/launch-demo.fasta` to show per-record
   biological kind validation.
5. Run `biors tokenize examples/launch-demo.fasta` to show stable protein-20
   token IDs.
6. Run `biors model-input --max-length 32 examples/launch-demo.fasta` to show
   `input_ids`, `attention_mask`, and truncation metadata.
7. Run `biors package verify examples/protein-package/manifest.json
   examples/protein-package/observations.json` to show a portable package
   fixture check.

## Contributor Demo

The contributor path is:

```bash
scripts/check-fast.sh
sh scripts/launch-demo.sh --cargo
```

This keeps the first contribution loop grounded in existing checks and a visible
research workflow.

## Benchmark Visual Draft

Use the committed benchmark report as the source for a simple three-bar chart:

- parse: bio-rs `44.73ms`, Biopython `197.66ms`
- validate: bio-rs `35.16ms`, Biopython `603.45ms`
- tokenize: bio-rs `57.97ms`, Biopython `598.02ms`

Chart title: `Matched FASTA workloads: bio-rs vs Biopython`

Caption: `Measured with the committed hyperfine benchmark artifact. Claims are
limited to these FASTA workloads and this recorded environment.`

## Browser Playground Concept

The no-install playground should start as a single browser flow:

1. Upload or paste FASTA.
2. Validate sequence kind.
3. Tokenize with `protein-20`.
4. Show JSON output in the same success envelope shape as the CLI.

The browser build must reuse `biors-core`; it should not fork validation or
tokenization behavior in JavaScript.

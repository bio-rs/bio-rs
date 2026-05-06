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
cargo install biors --version 0.20.0
biors --version
sh scripts/launch-demo.sh
```

## CLI Recorded Demo Script

Use `scripts/record-cli-demo.sh` when the output needs to be captured as a
terminal recording, GIF, or animated documentation asset:

```bash
sh scripts/record-cli-demo.sh --cargo
```

The script prints each command before running it, then emits the actual JSON
outputs from `biors`. It is deterministic for the committed dataset and avoids
checking in generated recordings.

An external recorder can wrap it without changing the repo:

```bash
asciinema rec -c "sh scripts/record-cli-demo.sh --cargo" bio-rs-demo.cast
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

- parse: bio-rs `47.04ms`, Biopython `208.10ms`
- validate: bio-rs `36.32ms`, Biopython `584.23ms`
- tokenize: bio-rs `60.62ms`, Biopython `586.54ms`

Chart title: `Matched FASTA workloads: bio-rs vs Biopython`

Caption: `Measured with the committed FASTA benchmark artifact. Claims are
limited to these workloads and this recorded environment.`

## Browser Playground Concept

Browser implementation is intentionally deferred for this release pass. The
concept remains:

1. Upload or paste FASTA.
2. Validate sequence kind.
3. Tokenize with `protein-20`.
4. Show JSON output in the same success envelope shape as the CLI.

The browser build must reuse `biors-core`; it should not fork validation or
tokenization behavior in JavaScript.

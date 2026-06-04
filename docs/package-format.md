# bio-rs Package Format

This document defines the portable package layout and manifest v1 contract.

## Directory Layout

A bio-rs package is a directory that can be copied, archived, checked into a
fixture repo, or resolved from a local artifact store without changing paths.
All paths in the manifest are package-relative and must stay inside the package
root.

```txt
protein-package/
  manifest.json
  models/
    protein-seed.onnx
  tokenizers/
    protein-20.json
  vocabs/
    protein-20.json
  pipelines/
    protein.toml
  fixtures/
    tiny.fasta
    tiny.output.json
  observed/
    tiny.output.json
  docs/
    LICENSE.txt
    CITATION.cff
    model-card.md
```

The `package_layout` manifest section declares the expected directory names:

```json
{
  "package_layout": {
    "manifest": "manifest.json",
    "models": "models",
    "tokenizers": "tokenizers",
    "vocabs": "vocabs",
    "pipelines": "pipelines",
    "fixtures": "fixtures",
    "observed": "observed",
    "docs": "docs"
  }
}
```

For v1 packages, `package_layout.manifest` must match the manifest file path
being validated relative to the package root. The default is `manifest.json`;
alternate names are allowed only when validation is run against that declared
manifest path.

`biors package validate` rejects absolute paths, `..` traversal, missing files,
checksum mismatches, and v1 assets that are outside their declared layout
directory.

## Manifest v1

Manifest v1 uses `schema_version: "biors.package.v1"` and is described by
`schemas/package-manifest.v1.json`.
Version support and migration rules are defined in
[Versioning Policy](versioning.md).

Required v1 sections:

- `package_layout`: portable directory contract for models, tokenizers, vocab,
  fixtures, observations, and package docs.
- `metadata.license`: SPDX-style license expression plus an optional package
  license file.
- `metadata.citation`: preferred citation text, optional DOI, and optional
  citation file.
- `metadata.model_card`: model card path, checksum, summary, intended use, and
  limitations.
- `model`, `preprocessing`, `postprocessing`, `runtime`, and `fixtures`: the
  executable package contract already present in v0.
- `runtime.version`: backend contract/version string pinned by generated
  pipeline lockfiles when package context is supplied.

Package metadata is intentionally explicit. A researcher should be able to
inspect the package and know whether the artifact can be redistributed, how to
cite it, and what the model card says before running inference.

## Model Artifact Metadata

Package manifests can attach optional model artifact metadata directly to the
`model` section:

```json
{
  "model": {
    "format": "onnx",
    "path": "models/protein-seed.onnx",
    "checksum": "sha256:...",
    "metadata": {
      "name": "protein-seed-linear-probe",
      "version": "fixture-0",
      "architecture": "linear-probe",
      "task": "classification",
      "source": "local-fixture",
      "description": "Tiny deterministic package fixture model."
    }
  }
}
```

`metadata.name` is required when the metadata object is present. The remaining
fields are optional and are intended for inspect output, bridge planning, and
future package registry surfaces. They are descriptive metadata, not execution
instructions.

## Runtime Compatibility

`biors package bridge` now reports deterministic compatibility checks between
the model artifact format and the declared runtime backend/target pair. Current
planning pairs are:

| Model format | Runtime backend | Runtime target | Execution provider |
| --- | --- | --- | --- |
| `onnx` | `onnx-webgpu` | `browser-wasm-webgpu` | `webgpu` |
| `safetensors` | `candle` | `local-cpu` | `candle-cpu` |

An incompatible pair, such as `onnx` with `candle/local-cpu`, produces a
blocking issue in the bridge report. This is a compatibility contract only; it
does not launch a browser, start a service, or link the optional Candle backend
into the default CLI binary.

`package bridge` keeps the legacy `ready` field as a compatibility alias for
`contract_ready`: manifest validation passed and the declared model/runtime pair
is supported. It also emits `artifact_checked`, `execution_ready`, and
`readiness_notes`. Current bridge planning does not parse model artifact bytes
or run an execution smoke test, so placeholder or corrupted artifacts can still
be `contract_ready: true` while `artifact_checked: false` and
`execution_ready: false`.

`biors package init` infers these defaults from the model filename extension:
`.onnx` selects `onnx` with `onnx-webgpu/browser-wasm-webgpu`, and
`.safetensors` selects `safetensors` with `candle/local-cpu`. Unknown model
extensions are rejected before a manifest is written.

For generated skeleton metadata, `package init` keeps the SPDX expression in
manifest metadata and writes `docs/LICENSE-SPDX.txt`; it writes free-form
preferred citation text to `docs/CITATION.txt`. This is a starter scaffold, not
a publication-quality metadata review. Use `package convert` with an explicit
`--citation-file docs/CITATION.cff` when supplying a validated Citation File
Format document. When `package init --tokenizer-config` is supplied, the
generated pipeline config uses the tokenizer profile's sequence kind and profile
instead of hardcoding the default protein profile.

Preprocessing steps may also reference a checked pipeline config artifact:

```json
{
  "name": "protein_fasta_tokenize",
  "implementation": "biors-core",
  "contract": "protein-20",
  "contract_version": "protein-20.v0",
  "config": {
    "path": "pipelines/protein.toml",
    "schema_version": "biors.pipeline.v0"
  }
}
```

When a step declares `config`, the path is package-relative and should live
under the declared `package_layout.pipelines` directory. CLI package validation
also parses the referenced config with the same validator used by
`biors pipeline --config`, so malformed config files fail package validation
even when the file exists and its checksum matches.
For package-declared pipeline configs, `input.path` must be relative and must
resolve inside the package root.

When a manifest declares a tokenizer asset, package validation parses the JSON
as a bio-rs tokenizer config and requires `tokenizer.name` and
`tokenizer.contract_version` to match the config profile.
When a manifest declares a vocab asset, package validation parses the JSON as a
bio-rs `Vocabulary`; the `protein-20` vocab must match the built-in token order,
token IDs, unknown token ID, and unknown-token policy.

Tokenizer/vocab asset names and contract versions, plus preprocessing and
postprocessing step names, implementations, contracts, and contract versions,
must be non-empty when present. Empty contract identifiers are rejected by both
the Rust manifest validator and the published JSON schemas.

Fixture names and declared `expected_input.shape` / `expected_output.shape`
dimensions must also contain non-whitespace text so JSON schema preflight
matches the Rust validator. Numeric dimensions and symbolic dimensions such as
`batch`, `sequence`, or `features` remain valid.

## Migration And Compatibility

`biors package migrate <manifest|-> --to biors.package.v1` emits an inspectable
migration plan instead of rewriting the manifest in place. The v0 to v1 path is
not automatic because v1 requires package layout and research metadata that
must be supplied by the package author.

`biors package convert <manifest|-> --to biors.package.v1` emits a converted
manifest and can write it with `--output <manifest.json>`. Conversion preserves
the v0 model, tokenizer, vocab, preprocessing, runtime, shape, and fixture
contracts, infers layout directories from existing package-relative paths, and
requires explicit metadata options for license, citation, model card,
intended-use, and limitations.

`biors tokenizer convert-hf`, `biors package init`, and
`biors package convert-project` cover the Python/Hugging Face project path.
They write package layout, tokenizer config, pipeline config, docs, fixture
references, and checksums while leaving optional `model.metadata` for the
package author to fill in. See [Package Conversion](package-conversion.md).

`biors package compatibility <left-manifest> <right-manifest>` reports the
schema transition from left to right, whether a migration is required, and
whether both manifests describe the same package name.

`biors package diff <left-manifest> <right-manifest>` combines that schema
context with a canonical manifest content diff. JSON manifests are compared as
parsed JSON values, so formatting-only changes do not produce content
mismatches.

## Fixture Verification Mismatches

`biors package verify` checks declared fixture outputs against observed outputs.
It verifies preprocessing and package fixture contracts; it does not run model
inference.

The example package includes a deliberate mismatch observation file:

```bash
biors package verify \
  examples/protein-package/manifest.json \
  --observations examples/protein-package/observations.mismatch.json \
  --json
```

That command is expected to fail with `package.output_content_mismatch`. Inspect
`error.details.results[0].diff.fields` to see the field-level mismatch between
the declared expected fixture output and the observed output without committing
large generated JSON snapshots.

## Checksums

Package artifacts use byte-for-byte `sha256:<64 lowercase hex>` checksums.
Validation computes raw file hashes from disk for:

- model artifacts
- tokenizer configs
- vocab files
- fixture inputs
- fixture expected outputs
- license files
- citation files
- model cards
- pipeline configs

Fixture verification reports raw expected/observed output checksums. JSON output
content is still compared canonically so formatting-only differences can be
identified separately from byte-level checksum drift.
Fixture names must be unique because verification observations use the fixture
name as their join key.

Checksums are optional for some fields in the schema so draft packages can be
assembled incrementally, but published or shared packages should include them
for every file that affects interpretation or reproducibility.

## Crate Split Review

The manifest contract remains in `biors-core` for the current pre-1.0 release
line instead of creating a new `biors-manifest` crate.

Rationale:

- The manifest types are still tightly coupled to package validation,
  verification, and runtime bridge planning.
- A separate crate would add public release coordination before there are
  independent downstream manifest consumers.
- The implementation is already isolated under
  `packages/rust/biors-core/src/package/`, so extracting it later can be done
  without changing package JSON.

Revisit the split when another Rust crate needs manifest parsing without the
rest of `biors-core`, or when manifest versioning becomes independent of core
sequence and tokenizer contracts.

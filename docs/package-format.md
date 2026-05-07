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
    "fixtures": "fixtures",
    "observed": "observed",
    "docs": "docs"
  }
}
```

`biors package validate` rejects absolute paths, `..` traversal, missing files,
checksum mismatches, and v1 assets that are outside their declared layout
directory.

## Manifest v1

Manifest v1 uses `schema_version: "biors.package.v1"` and is described by
`schemas/package-manifest.v1.json`.
Version support and migration rules are defined in
[Schema Versioning Policy](schema-versioning.md).

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

Package metadata is intentionally explicit. A researcher should be able to
inspect the package and know whether the artifact can be redistributed, how to
cite it, and what the model card says before running inference.

## Checksums

Package artifacts use `sha256:<64 lowercase hex>` checksums. Validation computes
hashes from disk for:

- model artifacts
- tokenizer configs
- vocab files
- fixture inputs
- fixture expected outputs
- license files
- citation files
- model cards

Checksums are optional for some fields in the schema so draft packages can be
assembled incrementally, but published or shared packages should include them
for every file that affects interpretation or reproducibility.

## Crate Split Review

The manifest contract remains in `biors-core` for v0.31.0 instead of creating a
new `biors-manifest` crate.

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

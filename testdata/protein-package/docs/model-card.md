# Protein Seed Fixture Model Card

This fixture is a tiny deterministic package used to test bio-rs package
layout, manifest metadata, and checksum validation.

## Intended Use

- Validate bio-rs package tooling in local and CI environments.
- Exercise package fixture verification without downloading external assets.

## Limitations

- The ONNX file is a placeholder artifact and is not a biological model.
- The fixture must not be used for scientific inference.

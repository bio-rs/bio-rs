# BIORS-CORE KNOWLEDGE BASE

## OVERVIEW

`biors-core` owns deterministic biological records, parsers, validators,
package/runtime contracts, reports, and model-ready input mapping.

## STRUCTURE

```text
src/
|-- fasta*.rs       # FASTA parsing, validation, inspection, streaming scan
|-- formats/        # FASTQ and shared format records/capabilities
|-- sequence/       # sequence kind, normalization, residue validation
|-- molecule/       # SMILES/SDF/MOL2 records, graph, validation, features
|-- structure/      # PDB records, coordinates, residue handling
|-- package/        # package manifests, layout, validation, artifacts
|-- runtime/        # runtime contracts and external-process bridge
|-- verification/   # fixture verification, diffs, hashes
`-- reports/        # JSON-to-human report assembly
```

## WHERE TO LOOK

| Task | Location | Notes |
| --- | --- | --- |
| Public exports | `src/lib.rs` | Keep surface explicit |
| Package validation | `src/package/validation*`, `tests/package_*` | Issue codes and path rules are contract behavior |
| Fixture verification | `src/verification/`, `tests/fixture_contract.rs` | Hashes and output comparison must be reproducible |
| Format parsers | `src/formats`, `src/fasta*` | Preserve diagnostics and record positions |
| Molecule/structure support | `src/molecule`, `src/structure` | Conservative validation over optimistic inference |
| Runtime bridge | `src/runtime`, `tests/runtime*` | No hidden process/network behavior |

## CONVENTIONS

- Keep code deterministic, side-effect-light, and usable from CLI, Python,
  WASM, MCP, and service surfaces.
- Use structured error types and stable issue/error codes when behavior crosses
  a public boundary.
- Package and pipeline validators must reject ambiguous paths, unknown schema
  versions, missing artifacts, checksum mismatches, and layout escapes.
- Fixtures in `tests/fixtures/` are crate-local; use root `testdata/` for
  end-to-end package or CLI fixtures.
- Prefer small modules by biological responsibility. Split before files become
  mixed parser/validator/renderer/tooling units.
- Do not use network access, filesystem writes outside explicit validation
  targets, clocks, randomness, or global mutable state in core logic unless a
  public contract requires it.

## CHECKS

```bash
cargo test -p biors-core
cargo check -p biors-core --target wasm32-unknown-unknown
cargo clippy -p biors-core --all-targets -- -D warnings
```

For package, runtime, or fixture contract changes, add the focused integration
test first, then run the root fast gate.

## ANTI-PATTERNS

- CLI formatting, Python class ergonomics, WASM serialization quirks, or MCP
  transport details inside core.
- Permissive parsing that drops diagnostics, positions, chain/residue identity,
  molecule bond context, or provenance needed by researchers.
- Future-facing feature names without implemented validation and tests.

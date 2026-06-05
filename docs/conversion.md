# Unified Conversion Layer

Version: 0.57.0

The conversion layer maps parsed biological records into one JSON-ready
`BioEntity` contract. It is local-only and deterministic: it does not call
models, rank results, upload data, or infer unsupported formats.

## Scope

Supported conversions:

- FASTA records -> kind-aware sequence entities
- FASTQ records -> DNA sequence entities with quality strings and source
  locations
- `StructureRecord` -> structure entities with extracted chain sequences
- SMILES/SDF/MOL2 `MoleculeRecord` values -> molecule entities with shared
  `FormatRecord` projection and deterministic derived features

`StructureRecord` is format-aware, so mmCIF records can use this conversion
contract once a mmCIF parser produces the same structure type. In 0.57.0, the
checked-in executable structure parser remains PDB.

## JSON Export

`BioEntityJsonExport` uses schema version `biors.conversion.v0` and contains:

- aggregate `records`, `valid_records`, `model_ready_records`,
  `warning_count`, and `error_count`
- `entities`, each with `id`, `entity_type`, `source`, `record`, and
  `validation`
- per-entity warnings/errors with stable conversion issue codes

The checked-in schema is `schemas/bio-entity-export-output.v0.json`.

## Model Readiness

The layer marks an entity `model_ready=true` only when conversion validation has
no warnings or errors. Sequence conversion runs the existing sequence validator,
rejects empty sequences, and checks FASTQ quality length. Structure and molecule
conversion reuse their format-specific validation reports and carry source issue
codes into conversion issues.

This contract prepares records for caller-owned model input mapping. It does not
execute tokenization, embeddings, inference, molecular docking, alignment, or
search ranking.

## Rust API

Key entrypoints:

- `convert_fasta_records(records, SequenceKindSelection::Auto)`
- `convert_fastq_records(records)`
- `structure_record_to_bio_entity(record)`
- `molecule_record_to_bio_entity(record)`
- `convert_molecule_records(records)`
- `export_bio_entities(entities)`

The conversion module reuses existing core types instead of inventing parallel
record shapes: `SequenceRecord`, `StructureRecord`, `MoleculeRecord`,
`FormatRecord`, and validation issue types remain the source of truth.

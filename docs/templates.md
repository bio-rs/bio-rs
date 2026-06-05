# Task Templates

Version: 0.55.0

Task templates are local, deterministic contracts for common bio-AI workflow
families. They describe required inputs, validations, model-ready fields,
expected outputs, and execution boundaries. They do not run inference, upload
data, open network connections, rank search results, or choose a model for the
caller.

## CLI

```bash
biors templates list
biors templates show molecule-property-prediction-v0
```

`templates list` emits `schemas/task-template-catalog-output.v0.json` inside
the standard CLI success envelope. `templates show <id>` emits
`schemas/task-template-output.v0.json`.

Unknown template ids fail with `template.not_found` in JSON error mode.

## Template Catalog

The stable template ids are:

| Template | Purpose |
| --- | --- |
| `protein-classification-v0` | Protein FASTA records to classification model inputs and scored labels |
| `protein-embedding-generation-v0` | Protein FASTA records to embedding input tensors and vectors |
| `variant-effect-prediction-v0` | Protein reference plus variant table fields to effect-score inputs |
| `molecule-property-prediction-v0` | SMILES/SDF/MOL2 molecule graphs to property-prediction features |
| `structure-validation-v0` | PDB structure records to validation and chain-summary outputs |
| `sequence-similarity-preprocess-v0` | FASTA/FASTQ records to normalized caller-side search preprocessing |

## Execution Boundary

Every template carries the same execution assumptions:

- `execution_mode`: `local_template_contract`
- `network_access`: `none`
- `external_model_calls`: `false`
- `uploads_input_data`: `false`
- `persistence`: `caller_controlled`

This keeps the templates useful for production pipelines without implying that
bio-rs owns model inference, search ranking, hosted processing, or storage.

## Format Support Boundary

Template inputs mark format support with `core_reader`:

- `executable`: bio-rs has a checked-in parser or validator for the format.
- `contract_only`: the template defines normalized fields, but bio-rs does not
  claim an executable parser for that format in this release.

The variant-effect template uses CSV/TSV as `contract_only` until a table parser
is introduced. Structure templates use executable PDB support; mmCIF remains a
reviewed structure candidate rather than a template input format in 0.55.0.

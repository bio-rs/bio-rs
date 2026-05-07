# Dataset Inputs And Artifact Store

Dataset inspection gives pipeline authors a reproducible view of local FASTA
inputs before validation or package conversion.

## Dataset Descriptor

```bash
biors dataset inspect \
  --source uniprot \
  --version 2026_02 \
  --split train \
  --metadata organism=human \
  ./datasets/human.fasta
```

The descriptor fields are:

- `source`: data origin such as `local`, `uniprot`, or an internal dataset name
- `version`: dataset release, snapshot, or commit identifier
- `split`: training/evaluation split name

Metadata is supplied as repeatable `--metadata key=value` pairs.

## Provenance

The output includes:

- resolved FASTA files
- file byte counts
- file SHA-256 hashes
- FASTA record counts
- `dataset_hash` over descriptor, metadata, files, and sample mapping
- `samples[]` mapping each FASTA record ID to dataset descriptor, source file,
  record index, file SHA-256, and sequence length

This mapping is intentionally local and deterministic. It does not upload or
resolve biological data through external services.

## Local Artifact Store

The default local artifact store root is:

```txt
.biors/artifacts
```

Set `BIORS_ARTIFACT_STORE` or pass `--root` to inspect another root.

The draft store layout is:

```txt
.biors/artifacts/
  packages/
  datasets/
  locks/
```

- `packages/`: resolved bio-rs package directories or unpacked archives
- `datasets/`: dataset snapshots keyed by source, version, split, and content hash
- `locks/`: `pipeline.lock` and provenance records for reproducible runs

Inspect the store:

```bash
biors cache inspect
```

Review cleanup without deleting:

```bash
biors cache clean --dry-run
```

Delete files only when the target root has been inspected:

```bash
biors cache clean --yes
```

`cache clean` rejects broad roots and requires either `--dry-run` or `--yes`.

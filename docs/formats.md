# Biological Format Support

Version: 0.47.16

This document records the `0.48.0` format-expansion contract. bio-rs keeps
format support local-first: parsing and validation run on local input, emit
machine-readable diagnostics, and do not upload biological data.

## Current Support Matrix

Run the machine-readable support matrix with:

```bash
biors formats list
```

| Format | Status | Current contract |
| --- | --- | --- |
| FASTA | Supported | Existing sequence parser, `SequenceRecord`, sequence validation, tokenization, and model-input surfaces |
| FASTQ | Supported | `FastqRecord`, `FastqValidationReport`, and shared `FormatRecord` projection |
| GFF3 | Reviewed candidate | Feature-row requirements documented; parser not exposed yet |
| GTF | Reviewed candidate | GTF attribute semantics documented separately from GFF3 |
| BED | Reviewed candidate | Coordinate requirements documented; parser not exposed yet |
| VCF | Reviewed candidate | Variant-row/header requirements documented; parser not exposed yet |
| GenBank | Reviewed candidate | Flat-file record and feature-table requirements documented; parser not exposed yet |
| UniProt flat file | Reviewed candidate | Accession, feature, taxonomy, and sequence requirements documented; parser not exposed yet |
| CSV biological table | Reviewed candidate | Header/row validation requirements documented; parser not exposed yet |
| TSV biological table | Reviewed candidate | Header/row validation requirements documented; parser not exposed yet |

`reviewed_candidate` means the validation requirements are public but the parser
is not part of the executable contract yet. Callers must not treat those formats
as parsed or validated by bio-rs until their status changes to `supported`.

## Shared Format Records

The Rust API exposes `biors_core::formats::FormatRecord` and
`FormatMetadata` as the shared shape for later conversion work:

- `format`: stable format identifier such as `fastq`, `gff3`, or `vcf`
- `id`: record identifier from the source format
- `metadata.record_index`: zero-based input record index
- `metadata.line_start` and `metadata.line_end`: one-based source line range
- `fields[]`: format-normalized name/value pairs

Format-specific records can project into this shape. For `FASTQ`,
`FastqRecord::to_format_record()` emits `sequence`, `quality`, and optional
`description` fields while keeping the richer `FastqRecord` available for
format-specific workflows.

## FASTQ Validation

Validate FASTQ reads with:

```bash
biors formats validate --format fastq reads.fastq
cat reads.fastq | biors formats validate --format fastq -
```

FASTQ support accepts multi-line sequence and quality bodies. It validates:

- header lines start with `@` and include a non-empty identifier
- separator lines start with `+`
- optional separator identifiers match the header identifier
- sequence bodies are non-empty
- sequence symbols are validated with the DNA IUPAC policy
- ambiguous DNA symbols such as `N` are reported as warnings
- unsupported sequence symbols are reported as errors
- quality string length exactly matches normalized sequence length
- quality symbols are printable Phred+33 ASCII characters (`!` through `~`)

The validation report deliberately does not repeat raw sequence and quality
payloads. It emits per-record IDs, descriptions, source line ranges, sequence
length, quality length, warning counts, error counts, and stable issue codes.

Top-level FASTQ parse failures use `fastq.*` error codes. Per-record biological
issues reuse the stable sequence issue codes where possible:

- `ambiguous_symbol`
- `invalid_symbol`
- `invalid_quality_character`

## Candidate Format Requirements

### GFF3

Before GFF3 becomes `supported`, the parser must validate:

- exactly nine tab-delimited columns
- one-based inclusive coordinates with `start <= end`
- strand and phase enumerations
- URL-decoded attributes with an explicit duplicate-key policy
- GFF3 attribute semantics independently from GTF

### GTF

Before GTF becomes `supported`, the parser must validate:

- exactly nine tab-delimited columns
- one-based inclusive coordinates with `start <= end`
- quoted attribute parsing
- `gene_id` and `transcript_id` preservation when present
- strand and frame enumerations

### BED

Before BED becomes `supported`, the parser must validate:

- at least three tab-delimited columns
- zero-based half-open intervals with `start < end`
- score range when a score column is present
- strand enumeration when a strand column is present

BED coordinates must not be normalized into GFF/GTF coordinates silently.

### VCF

Before VCF becomes `supported`, the parser must validate:

- header metadata lines and the `#CHROM` column line
- one-based `POS`
- non-empty `REF`
- `ALT` allele lists and symbolic alleles
- `INFO` key/value parsing with a duplicate-key policy
- genotype/sample columns in a separate schema pass

### GenBank

Before GenBank becomes `supported`, the parser must validate:

- `LOCUS`, `FEATURES`, `ORIGIN`, and `//` record boundaries
- sequence length parity between `LOCUS` and `ORIGIN`
- accession/version preservation
- feature location grammar, including `join` and `complement`

### UniProt Flat File

Before UniProt flat file support becomes `supported`, the parser must validate:

- `ID`, `AC`, `DE`, `OS`, `SQ`, and `//` sections
- sequence length parity with `SQ` metadata
- primary and secondary accession preservation
- feature table coordinate ranges

### CSV and TSV Biological Tables

Before table parsing becomes `supported`, bio-rs must validate:

- a header row with non-empty, unique column names
- stable delimiter and newline behavior
- row-width parity
- quoted field handling for CSV before biological validation
- caller-selected biological columns for sequence, variant, or entity
  validation

TSV can land before CSV if the CSV quoted-field contract is still under review.

## `crates/biors-formats` Decision

`0.48.0` does not introduce a separate `biors-formats` crate. The supported
FASTQ parser and shared format contracts live in `biors-core::formats` because:

- the release has one supported new format family, so a new crate would add
  packaging and release surface before there is a stable split point
- `FormatRecord` is needed by later core conversion work in `0.51.0`
- keeping the API in `biors-core` lets Rust, CLI, Python, WASM, MCP, and service
  surfaces converge on one contract first

A separate crate should be reconsidered once at least two non-sequence format
families are executable and the conversion layer can depend on a stable
format-only API without pulling in unrelated package/runtime code.

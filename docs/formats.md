# Biological Format Support

Version: 0.57.2

This document records the `0.57.2` format-expansion contract. bio-rs keeps
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
| PDB | Supported | `StructureRecord`, PDB chain extraction, coordinate validation, missing-residue preservation, and sequence mapping |
| mmCIF | Reviewed candidate | `_atom_site` and sequence-category requirements documented; parser not exposed yet |
| CSV biological table | Reviewed candidate | Header/row validation requirements documented; parser not exposed yet |
| TSV biological table | Reviewed candidate | Header/row validation requirements documented; parser not exposed yet |
| SMILES | Supported | `MoleculeRecord`, molecular graph construction, validation, descriptors, and fingerprints |
| SDF | Supported | V2000/basic V3000 molecule parsing with coordinates and SDF data-item preservation |
| MOL2 | Supported | TRIPOS molecule/atom/bond parsing with atom types, partial charges, and substructure metadata |

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

### PDB

PDB support is executable through `biors structure validate --format pdb` and
`biors structure sequence --format pdb`. The parser uses fixed-column
ATOM/HETATM records, records chains/residues/atoms, preserves `REMARK 465`
missing residues, validates coordinates and occupancy values, extracts
coordinate-derived protein sequences, and maps them against SEQRES when
present. See [Structure support](structure.md) for the detailed contract.

### mmCIF

Before mmCIF becomes `supported`, the parser must validate:

- `_atom_site` loops for atom identifiers, residue identifiers, chain
  identifiers, Cartesian coordinates, occupancy, temperature factor, and
  element symbol
- label/auth chain identifier normalization
- sequence categories and coordinate categories into the shared
  `StructureRecord`
- missing-residue categories into `MissingResidue`
- coordinate validation parity with PDB
- protein sequence and structure mapping parity with PDB

### CSV and TSV Biological Tables

Before table parsing becomes `supported`, bio-rs must validate:

- a header row with non-empty, unique column names
- stable delimiter and newline behavior
- row-width parity
- quoted field handling for CSV before biological validation
- caller-selected biological columns for sequence, variant, or entity
  validation

TSV can land before CSV if the CSV quoted-field contract is still under review.

### SMILES, SDF, And MOL2

Molecule support is executable through:

```bash
biors molecule validate --format smiles molecules.smi
biors molecule inspect --format sdf compounds.sdf
biors molecule validate --format mol2 ligand.mol2
```

All three formats project into `MoleculeRecord` with `AtomGraph`, `BondGraph`,
`MoleculeMetadata`, and preserved source properties. Validation includes graph
construction, disconnected component counting, conservative valence checks,
deterministic canonical graph keys, formula/mass descriptors, simple
drug-discovery descriptors, and `biors-ecfp-lite-v0` hashed fingerprints.

SMILES support covers organic subset atoms, bracket atoms, branches, bond
orders, directional bond markers, one-digit and `%NN` ring closures, and
disconnected components. SDF support covers V2000 connection tables, basic
V3000 atom/bond blocks, coordinates, and data items. MOL2 support covers
`@<TRIPOS>MOLECULE`, `ATOM`, and `BOND` sections, including atom types,
partial charges, and substructure metadata.

The canonical graph key is a bio-rs deterministic graph key, not an
RDKit/Open Babel canonical SMILES equivalence claim. Aromatic source notation
is preserved and flagged when aromaticity perception has not been independently
verified. See [Molecule support](molecule.md) for the detailed contract.

## `crates/biors-formats` Decision

`0.57.2` does not introduce a separate `biors-formats` crate. The supported
FASTQ parser, molecule parsers, and shared format contracts live in
`biors-core` because:

- the molecule contracts share validation and derived-feature code with the
  core conversion work planned for entity mapping
- `FormatRecord` is shared by the current conversion and format-capability
  contracts
- keeping the API in `biors-core` lets Rust, CLI, Python, WASM, MCP, and service
  surfaces converge on one contract first

A separate crate should be reconsidered once at least two non-sequence format
families are executable and the conversion layer can depend on a stable
format-only API without pulling in unrelated package/runtime code.

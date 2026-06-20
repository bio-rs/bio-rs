# Structure Support

Version: 0.57.4

This document records the structure-format contract for the current
development line. Structure parsing stays local-first: bio-rs reads local input
or stdin, emits JSON diagnostics, and does not upload structure files.

## Current Support Matrix

| Format | Status | Contract |
| --- | --- | --- |
| PDB | Supported | Fixed-column ATOM/HETATM parsing, chain extraction, missing-residue preservation, coordinate validation, and protein sequence mapping |
| mmCIF | Reviewed candidate | Requirements documented; parser not exposed yet |

`reviewed_candidate` means requirements are public but the parser is not an
executable bio-rs contract yet.

## PDB Commands

Validate a PDB file:

```bash
biors structure validate --format pdb structure.pdb
cat structure.pdb | biors structure validate --format pdb -
```

Extract coordinate-derived and SEQRES-derived protein sequences:

```bash
biors structure sequence --format pdb structure.pdb
```

Both commands emit the standard CLI success envelope with `input_hash`.
Validation output uses `schemas/structure-validation-output.v0.json`; sequence
extraction uses `schemas/structure-sequence-output.v0.json`.

## PDB Record Contract

The Rust API exposes:

- `StructureRecord`
- `StructureMetadata`
- `Chain`
- `Residue3D`
- `Atom`
- `Coordinate`
- `MissingResidue`
- `StructureValidationReport`
- `StructureSequenceOutput`
- `ProteinStructureMapping`

The parser reads PDB `ATOM` and `HETATM` records using the wwPDB fixed-column
coordinate layout. It records atom serial, atom name, alternate location,
residue name, chain ID, residue sequence number, insertion code, coordinates,
occupancy, temperature factor, and element symbol when present.

Blank PDB chain identifiers are normalized to `_` in JSON output so downstream
schemas do not need empty string special cases.

When a PDB file contains multiple `MODEL` blocks, bio-rs parses coordinates
from the first model only and reports the total model count in
`StructureMetadata`. This avoids silently merging multiple coordinate models
into one chain. Later releases can add explicit model-selection controls.

## Validation Requirements

PDB support validates:

- input is readable UTF-8 text
- required ATOM/HETATM fixed-column fields are present
- atom serials and residue sequence numbers parse as integers
- x/y/z coordinates parse as finite floating-point values
- occupancy and temperature factor values parse as finite values when present
- negative occupancy is an error
- occupancy greater than `1.0` is a warning
- missing element symbols are warnings
- `REMARK 465` missing residues are preserved as warnings
- coordinate-derived protein sequence can be mapped to SEQRES when SEQRES is
  present

Top-level parse failures use `pdb.*` error codes. Biological or structural
validation findings are reported inside successful validation payloads with
stable issue codes.

## Sequence Extraction and Mapping

`biors structure sequence --format pdb` returns one entry per chain:

- `coordinate_sequence`: protein sequence derived from coordinate-bearing
  non-HETATM residues
- `seqres_sequence`: protein sequence derived from SEQRES when present
- `missing_residues`: residues annotated by `REMARK 465`
- `mapping.coordinate_to_seqres_positions`: one-based SEQRES positions for
  each coordinate residue, or null values when the chain cannot be mapped

Mapping statuses:

- `exact`: coordinate sequence exactly matches SEQRES
- `coordinate_subsequence`: coordinate sequence is an ordered subsequence of
  SEQRES, commonly because unresolved residues are absent from coordinates
- `missing_seqres`: no SEQRES sequence was available for the chain
- `mismatch`: coordinate sequence cannot be mapped to SEQRES in order

Unknown or modified protein residues use `X` in extracted sequences and are
reported as warnings.

## mmCIF Reviewed Candidate Requirements

Before mmCIF becomes `supported`, bio-rs must freeze and test:

- `_atom_site` loop parsing for Cartesian coordinates
- label/auth chain identifier policy
- sequence category mapping into `StructureRecord`
- missing-residue category mapping into `MissingResidue`
- coordinate validation parity with PDB
- protein sequence and structure mapping parity with PDB
- JSON output parity with the PDB structure schemas

The reviewed mmCIF target is the same `StructureRecord` contract rather than a
separate output family.

## Sources

The PDB parser follows the wwPDB PDB format 3.3 coordinate section for
`MODEL`, `ATOM`, and `HETATM` records:

- https://www.wwpdb.org/documentation/file-format-content/format33/sect9.html

SEQRES and missing-residue requirements are aligned with the wwPDB primary
structure section and coordinate-section sequence relationship notes:

- https://www.wwpdb.org/documentation/file-format-content/format33/sect3.html

mmCIF requirements use the official PDBx/mmCIF dictionary categories, starting
with `_atom_site`:

- https://mmcif.wwpdb.org/dictionaries/mmcif_pdbx_v50.dic/Categories/atom_site.html

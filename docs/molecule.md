# Molecule Support

Version: 0.57.1

bio-rs molecule support is local-first and privacy-first. SMILES, SDF, and MOL2
parsing run on local input and emit JSON contracts; bio-rs does not upload
molecule data.

## Commands

```bash
printf 'CC(=O)O acetate\n' | biors molecule validate --format smiles -
biors molecule inspect --format smiles molecules.smi
biors molecule validate --format sdf compounds.sdf
biors molecule inspect --format mol2 ligand.mol2
```

`validate` emits `schemas/molecule-validation-output.v0.json`. `inspect` emits
`schemas/molecule-records-output.v0.json`.

## Record Contract

All molecule formats project into `MoleculeRecord`:

- `format`: `smiles`, `sdf`, or `mol2`
- `id`: optional source identifier or molecule title
- `source`: source SMILES token, SDF title, or MOL2 molecule name
- `metadata`: `MoleculeMetadata`
- `graph`: split `AtomGraph` and `BondGraph`
- `properties`: SDF data items and MOL2 metadata sections preserved as
  name/value pairs

`MoleculeMetadata` records source line metadata, atom count, bond count, branch
count, completed ring closures, disconnected component count, and aromatic atom
count.

`MoleculeAtom` preserves element, original token/name, aromatic/bracket flags,
isotope, explicit hydrogens, formal charge, chirality, atom class, optional 3D
coordinate, optional MOL2 atom type, partial charge, and substructure metadata.

`MoleculeBond` preserves source and target atom indices, bond order, ring
closure marker, and optional directional stereochemistry marker.

## Supported Parsing

SMILES support covers the OpenSMILES core syntax used by common bio-AI and
screening pipelines:

- organic subset atoms and wildcard atoms
- bracket atoms with isotope, hydrogens, charge, chirality marker, and atom
  class
- single, double, triple, quadruple, aromatic, and directional bond markers
- branches
- one-digit and `%NN` ring closures
- disconnected components
- optional line-level record ids

SDF support covers V2000 connection tables, basic V3000 atom/bond blocks, and
SDF data items. It preserves atom coordinates and data-item values so assay or
property columns are not silently discarded.

MOL2 support covers `@<TRIPOS>MOLECULE`, `ATOM`, and `BOND` sections. It
preserves atom coordinates, atom types, partial charges, and substructure
fields used by docking workflows.

## Validation And Features

Molecule validation includes:

- branch stack and ring-closure balance
- bracket atom syntax
- SDF counts/atom/bond block validation
- MOL2 molecule/atom/bond section validation
- disconnected component counting
- conservative valence checks for common organic and bioactive atoms
- deterministic topology-oriented `canonical_graph_key`
- explicit-atom empirical formula, including only hydrogens represented in the
  parsed source
- explicit-atom exact mass for common bioactive elements
- heavy atom, hetero atom, ring bond, rotatable bond, donor, acceptor, and
  formal charge descriptors
- deterministic `biors-ecfp-lite-v0` hashed fingerprint

Warnings do not make a record invalid. `valid=false` is reserved for syntax or
chemical errors such as conservative valence overflow.

## Boundaries

The `canonical_graph_key` is a deterministic bio-rs graph key for equality,
deduplication, and fixture comparison. It is not a claim of RDKit/Open Babel
canonical SMILES equivalence, and it does not encode stereochemical or isotope
equivalence classes beyond the parsed atom tokens preserved in the record.

bio-rs preserves aromatic source notation and emits an
`aromaticity_not_verified` warning when aromatic notation is present. Use a
specialized cheminformatics toolkit when a workflow requires tautomer
normalization, aromaticity perception, exhaustive stereochemistry assignment,
3D conformer generation, or force-field chemistry.

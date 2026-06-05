//! Molecule parsing, graph representation, and validation APIs.

mod error;
mod features;
mod graph;
pub mod mol2;
pub mod sdf;
pub mod smiles;
mod types;
mod validation;

pub use error::MoleculeReadError;
pub use features::derive_molecule_features;
pub use mol2::{parse_mol2_records, parse_mol2_records_reader, Mol2ParseError};
pub use sdf::{parse_sdf_records, parse_sdf_records_reader, SdfParseError};
pub use smiles::{
    parse_smiles_records, parse_smiles_records_reader, validate_smiles_reader,
    validate_smiles_reader_with_hash, SmilesParseError,
};
pub use types::{
    AtomGraph, BondGraph, BondOrder, MolecularGraph, MoleculeAtom, MoleculeBond,
    MoleculeCoordinate, MoleculeDerivedFeatures, MoleculeFingerprint, MoleculeMetadata,
    MoleculeProperty, MoleculeRecord, MoleculeValidationIssue, MoleculeValidationIssueCode,
    MoleculeValidationRecord, MoleculeValidationReport, ParsedMoleculeInput,
    ValidatedMoleculeInput,
};
pub use validation::{summarize_molecule_records, validate_molecule_records};

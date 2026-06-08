use super::super::{FormatCapability, FormatSupportStatus};
use crate::formats::BioFormat;

pub(super) fn capabilities() -> Vec<FormatCapability> {
    vec![
        FormatCapability {
            format: BioFormat::Smiles,
            status: FormatSupportStatus::Supported,
            record_contract:
                "MoleculeRecord with AtomGraph, BondGraph, and MoleculeMetadata".to_string(),
            validation_requirements: vec![
                "line-oriented SMILES token parsing with optional record identifier".to_string(),
                "branch stack and ring-closure balance validation".to_string(),
                "bracket atom isotope, hydrogen, charge, chirality, and atom-class parsing"
                    .to_string(),
                "molecular graph construction with explicit bond order and disconnected components"
                    .to_string(),
                "conservative valence validation for common organic and bioactive atoms"
                    .to_string(),
            ],
            notes: vec![
                "aromatic source notation is preserved, but Huckel aromaticity assignment is reported as unverified".to_string(),
                "canonical graph keys, formula/mass descriptors, simple molecular descriptors, and biors-ecfp-lite-v0 fingerprints are computed from the parsed graph".to_string(),
                "canonical graph keys are not RDKit/Open Babel canonical SMILES equivalence claims".to_string(),
            ],
        },
        FormatCapability {
            format: BioFormat::Sdf,
            status: FormatSupportStatus::Supported,
            record_contract: "MoleculeRecord projection from V2000/V3000 connection tables"
                .to_string(),
            validation_requirements: vec![
                "record boundary parsing with $$$$ delimiters".to_string(),
                "counts-line, atom block, bond block, and property block validation".to_string(),
                "SDF data-item preservation without silently dropping assay columns".to_string(),
                "V2000 and basic V3000 atom/bond block parsing before molecule graph construction".to_string(),
            ],
            notes: vec![
                "V2000 connection tables and basic V3000 atom/bond blocks are executable; advanced CTfile stereochemistry is preserved only where source fields map to the public graph contract".to_string(),
            ],
        },
        FormatCapability {
            format: BioFormat::Mol2,
            status: FormatSupportStatus::Supported,
            record_contract: "MoleculeRecord projection from @<TRIPOS>ATOM and @<TRIPOS>BOND"
                .to_string(),
            validation_requirements: vec![
                "TRIPOS section boundary parsing".to_string(),
                "atom id/name/type/coordinate/charge preservation".to_string(),
                "bond id/source/target/type validation".to_string(),
                "substructure and charge fields retained for docking workflows".to_string(),
            ],
            notes: vec![
                "MOL2 atom types, partial charges, coordinates, and substructure metadata are preserved for docking-oriented preprocessing".to_string(),
            ],
        },
    ]
}

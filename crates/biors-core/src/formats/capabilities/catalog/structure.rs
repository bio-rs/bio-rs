use super::super::{FormatCapability, FormatSupportStatus};
use crate::formats::BioFormat;

pub(super) fn capabilities() -> Vec<FormatCapability> {
    vec![
        FormatCapability {
            format: BioFormat::Pdb,
            status: FormatSupportStatus::Supported,
            record_contract:
                "StructureRecord with Chain, Residue3D, Atom, Coordinate, and StructureMetadata"
                    .to_string(),
            validation_requirements: vec![
                "fixed-column ATOM/HETATM coordinate parsing".to_string(),
                "chain and residue extraction without merging MODEL blocks".to_string(),
                "REMARK 465 missing-residue preservation".to_string(),
                "finite coordinate and occupancy range validation".to_string(),
                "coordinate-derived protein sequence extraction and SEQRES mapping".to_string(),
            ],
            notes: vec![
                "PDB parser consumes the first MODEL block when multiple models are present"
                    .to_string(),
                "blank PDB chain identifiers are normalized to '_' in JSON output".to_string(),
            ],
        },
        FormatCapability {
            format: BioFormat::Mmcif,
            status: FormatSupportStatus::ReviewedCandidate,
            record_contract: "StructureRecord projection from _atom_site and sequence categories"
                .to_string(),
            validation_requirements: vec![
                "_atom_site Cartesian coordinate parsing".to_string(),
                "auth/label chain identifier normalization policy".to_string(),
                "entity_poly_seq and missing-residue category mapping".to_string(),
                "coordinate validation parity with PDB".to_string(),
                "protein sequence and structure mapping parity with PDB".to_string(),
            ],
            notes: vec![
                "mmCIF remains reviewed until loop/category parsing semantics are frozen"
                    .to_string(),
            ],
        },
    ]
}

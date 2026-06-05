use serde::{Deserialize, Serialize};

use super::records::BioFormat;

/// Implementation state for a biological format in the current release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormatSupportStatus {
    /// Parser and validation contract are executable in the current release.
    Supported,
    /// Requirements are documented, but parser support is not exposed yet.
    ReviewedCandidate,
    /// Explicitly out of scope for the current release line.
    Future,
}

/// Public capability and validation-requirement summary for one format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormatCapability {
    /// Format family.
    pub format: BioFormat,
    /// Current support state.
    pub status: FormatSupportStatus,
    /// Shared or format-specific record contract.
    pub record_contract: String,
    /// Validation requirements that must be met before records become trusted.
    pub validation_requirements: Vec<String>,
    /// Non-contract notes for users and implementers.
    pub notes: Vec<String>,
}

/// Return the current format support matrix.
pub fn format_capabilities() -> Vec<FormatCapability> {
    vec![
        FormatCapability {
            format: BioFormat::Fasta,
            status: FormatSupportStatus::Supported,
            record_contract: "SequenceRecord / legacy ProteinSequence".to_string(),
            validation_requirements: vec![
                "non-empty header identifiers".to_string(),
                "non-empty sequence body per record".to_string(),
                "protein, DNA, RNA, or auto-detected alphabet validation".to_string(),
                "stable per-record warning/error diagnostics".to_string(),
            ],
            notes: vec!["existing sequence parser and tokenizer input surface".to_string()],
        },
        FormatCapability {
            format: BioFormat::Fastq,
            status: FormatSupportStatus::Supported,
            record_contract: "FastqRecord plus shared FormatRecord projection".to_string(),
            validation_requirements: vec![
                "header line starts with @ and includes a non-empty identifier".to_string(),
                "separator line starts with + and optional identifier matches the header"
                    .to_string(),
                "sequence body is non-empty and validates against DNA IUPAC policy".to_string(),
                "quality string length exactly matches normalized sequence length".to_string(),
                "quality symbols are printable Phred+33 ASCII characters".to_string(),
            ],
            notes: vec![
                "multi-line sequence and quality bodies are accepted".to_string(),
                "quality scores are validated structurally but not decoded into probabilities"
                    .to_string(),
            ],
        },
        FormatCapability {
            format: BioFormat::Gff3,
            status: FormatSupportStatus::ReviewedCandidate,
            record_contract:
                "FormatRecord with seqid/source/type/start/end/score/strand/phase/attributes"
                    .to_string(),
            validation_requirements: vec![
                "nine tab-delimited columns".to_string(),
                "one-based inclusive coordinates with start <= end".to_string(),
                "strand and phase enumerations".to_string(),
                "URL-decoded attribute keys with duplicate-key policy".to_string(),
            ],
            notes: vec![
                "GFF3 parser should stay separate from GTF attribute semantics".to_string(),
            ],
        },
        FormatCapability {
            format: BioFormat::Gtf,
            status: FormatSupportStatus::ReviewedCandidate,
            record_contract: "FormatRecord with GTF attributes preserving gene_id/transcript_id"
                .to_string(),
            validation_requirements: vec![
                "nine tab-delimited columns".to_string(),
                "one-based inclusive coordinates with start <= end".to_string(),
                "required quoted attribute parsing for gene_id and transcript_id when present"
                    .to_string(),
                "strand and frame validation".to_string(),
            ],
            notes: vec!["GTF parser must not silently apply GFF3 attribute rules".to_string()],
        },
        FormatCapability {
            format: BioFormat::Bed,
            status: FormatSupportStatus::ReviewedCandidate,
            record_contract: "FormatRecord with chrom/start/end/name/score/strand fields"
                .to_string(),
            validation_requirements: vec![
                "minimum three tab-delimited columns".to_string(),
                "zero-based half-open coordinates with start < end".to_string(),
                "score range validation when score is present".to_string(),
                "strand enumeration when strand is present".to_string(),
            ],
            notes: vec!["BED coordinate semantics intentionally differ from GFF/GTF".to_string()],
        },
        FormatCapability {
            format: BioFormat::Vcf,
            status: FormatSupportStatus::ReviewedCandidate,
            record_contract: "FormatRecord with CHROM/POS/ID/REF/ALT/QUAL/FILTER/INFO".to_string(),
            validation_requirements: vec![
                "header metadata and #CHROM column line parsing".to_string(),
                "one-based POS and non-empty REF allele".to_string(),
                "ALT allele list and symbolic allele handling".to_string(),
                "INFO key/value parsing with duplicate-key policy".to_string(),
            ],
            notes: vec!["genotype/sample columns require an additional schema pass".to_string()],
        },
        FormatCapability {
            format: BioFormat::Genbank,
            status: FormatSupportStatus::ReviewedCandidate,
            record_contract: "FormatRecord with feature table projection requirements"
                .to_string(),
            validation_requirements: vec![
                "LOCUS, FEATURES, ORIGIN, and terminator boundaries".to_string(),
                "sequence length parity between LOCUS and ORIGIN".to_string(),
                "feature location grammar with join/complement handling".to_string(),
                "accession/version metadata preservation".to_string(),
            ],
            notes: vec![
                "feature-location parsing should be tested before conversion support".to_string(),
            ],
        },
        FormatCapability {
            format: BioFormat::UniprotFlat,
            status: FormatSupportStatus::ReviewedCandidate,
            record_contract: "FormatRecord plus accession, taxonomy, feature, and sequence fields"
                .to_string(),
            validation_requirements: vec![
                "ID/AC/DE/OS/SQ sections and // record terminator".to_string(),
                "sequence length parity with SQ metadata".to_string(),
                "accession list preservation".to_string(),
                "feature table coordinate validation".to_string(),
            ],
            notes: vec![
                "FASTA export should preserve primary accession as the stable id".to_string(),
            ],
        },
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
        FormatCapability {
            format: BioFormat::Csv,
            status: FormatSupportStatus::ReviewedCandidate,
            record_contract: "FormatRecord with header-derived fields".to_string(),
            validation_requirements: vec![
                "header row required with non-empty unique column names".to_string(),
                "RFC-style quoted field handling before biological validation".to_string(),
                "stable delimiter and newline policy".to_string(),
                "caller-selected biological columns for sequence/variant/entity validation"
                    .to_string(),
            ],
            notes: vec![
                "CSV remains reviewed until a dependency-light parser contract is frozen"
                    .to_string(),
            ],
        },
        FormatCapability {
            format: BioFormat::Tsv,
            status: FormatSupportStatus::ReviewedCandidate,
            record_contract: "FormatRecord with header-derived fields".to_string(),
            validation_requirements: vec![
                "header row required with non-empty unique column names".to_string(),
                "tab-delimited row width parity".to_string(),
                "no silent trimming of biological field values".to_string(),
                "caller-selected biological columns for sequence/variant/entity validation"
                    .to_string(),
            ],
            notes: vec![
                "TSV parser can land before CSV if quoted-field support remains deferred"
                    .to_string(),
            ],
        },
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

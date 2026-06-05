use crate::formats::BioFormat;

use super::{TemplateEntity, TemplateInput, TemplateInputFormat};

static FASTA_FORMATS: [TemplateInputFormat; 1] =
    [TemplateInputFormat::executable(BioFormat::Fasta)];
static TABLE_FORMATS: [TemplateInputFormat; 2] = [
    TemplateInputFormat::contract_only(BioFormat::Csv),
    TemplateInputFormat::contract_only(BioFormat::Tsv),
];
static MOLECULE_FORMATS: [TemplateInputFormat; 3] = [
    TemplateInputFormat::executable(BioFormat::Smiles),
    TemplateInputFormat::executable(BioFormat::Sdf),
    TemplateInputFormat::executable(BioFormat::Mol2),
];
static PDB_FORMATS: [TemplateInputFormat; 1] = [TemplateInputFormat::executable(BioFormat::Pdb)];
static SEARCH_FORMATS: [TemplateInputFormat; 2] = [
    TemplateInputFormat::executable(BioFormat::Fasta),
    TemplateInputFormat::executable(BioFormat::Fastq),
];

static PROTEIN_INPUT_FIELDS: [&str; 3] = ["id", "sequence", "sequence_kind"];
static VARIANT_INPUT_FIELDS: [&str; 5] = [
    "variant_id",
    "sequence_id",
    "position",
    "reference_residue",
    "alternate_residue",
];
static MOLECULE_INPUT_FIELDS: [&str; 4] = ["id", "source", "format", "molecular_graph"];
static STRUCTURE_INPUT_FIELDS: [&str; 4] = ["id", "chains", "residues", "atoms"];
static SEARCH_INPUT_FIELDS: [&str; 4] = ["id", "sequence", "sequence_kind", "source_hash"];

pub(super) static PROTEIN_SEQUENCE_INPUT: [TemplateInput; 1] = [TemplateInput {
    entity: TemplateEntity::ProteinSequence,
    formats: &FASTA_FORMATS,
    required_fields: &PROTEIN_INPUT_FIELDS,
}];
pub(super) static VARIANT_INPUTS: [TemplateInput; 2] = [
    TemplateInput {
        entity: TemplateEntity::ProteinSequence,
        formats: &FASTA_FORMATS,
        required_fields: &PROTEIN_INPUT_FIELDS,
    },
    TemplateInput {
        entity: TemplateEntity::ProteinVariant,
        formats: &TABLE_FORMATS,
        required_fields: &VARIANT_INPUT_FIELDS,
    },
];
pub(super) static MOLECULE_INPUT: [TemplateInput; 1] = [TemplateInput {
    entity: TemplateEntity::Molecule,
    formats: &MOLECULE_FORMATS,
    required_fields: &MOLECULE_INPUT_FIELDS,
}];
pub(super) static STRUCTURE_INPUT: [TemplateInput; 1] = [TemplateInput {
    entity: TemplateEntity::ProteinStructure,
    formats: &PDB_FORMATS,
    required_fields: &STRUCTURE_INPUT_FIELDS,
}];
pub(super) static SEARCH_INPUT: [TemplateInput; 1] = [TemplateInput {
    entity: TemplateEntity::SequenceSet,
    formats: &SEARCH_FORMATS,
    required_fields: &SEARCH_INPUT_FIELDS,
}];

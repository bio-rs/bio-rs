//! Unified conversion records for model-facing biological entity payloads.

mod entity;
mod issue;
mod sequence;
mod types;

pub use entity::{
    convert_molecule_records, export_bio_entities, molecule_record_to_bio_entity,
    structure_record_to_bio_entity,
};
pub use sequence::{
    convert_fasta_records, convert_fastq_records, fasta_record_to_bio_entity,
    fastq_record_to_bio_entity,
};
pub use types::{
    BioEntity, BioEntityJsonExport, BioEntityType, ConversionIssue, ConversionIssueCode,
    ConversionIssueSeverity, ConversionRecord, ConversionSource, ConversionValidation,
    ConvertedMoleculeRecord, ConvertedSequenceRecord, ConvertedStructureRecord,
    CONVERSION_SCHEMA_VERSION,
};

use crate::molecule::{derive_molecule_features, validate_molecule_records, MoleculeRecord};
use crate::structure::{extract_structure_sequences, validate_structure_record, StructureRecord};

use super::issue::{molecule_error, molecule_warning, structure_error, structure_warning};
use super::types::{
    BioEntity, BioEntityJsonExport, BioEntityType, ConversionRecord, ConversionSource,
    ConversionValidation, ConvertedMoleculeRecord, ConvertedStructureRecord,
    CONVERSION_SCHEMA_VERSION,
};

/// Convert parsed molecule records into unified molecule entities.
pub fn convert_molecule_records(records: &[MoleculeRecord]) -> BioEntityJsonExport {
    export_bio_entities(
        records
            .iter()
            .map(molecule_record_to_bio_entity)
            .collect::<Vec<_>>(),
    )
}

/// Wrap converted entities in a JSON-ready export with aggregate counts.
pub fn export_bio_entities(entities: Vec<BioEntity>) -> BioEntityJsonExport {
    BioEntityJsonExport {
        schema_version: CONVERSION_SCHEMA_VERSION.to_string(),
        records: entities.len(),
        valid_records: entities
            .iter()
            .filter(|entity| entity.validation.valid)
            .count(),
        model_ready_records: entities
            .iter()
            .filter(|entity| entity.validation.model_ready)
            .count(),
        warning_count: entities
            .iter()
            .map(|entity| entity.validation.warning_count)
            .sum(),
        error_count: entities
            .iter()
            .map(|entity| entity.validation.error_count)
            .sum(),
        entities,
    }
}

/// Convert one parsed structure record into a unified structure entity.
pub fn structure_record_to_bio_entity(record: &StructureRecord) -> BioEntity {
    let report = validate_structure_record(record);
    let warnings = report
        .warnings
        .iter()
        .map(structure_warning)
        .collect::<Vec<_>>();
    let errors = report
        .errors
        .iter()
        .map(structure_error)
        .collect::<Vec<_>>();
    let model_ready = report.valid && warnings.is_empty() && errors.is_empty();
    let validation = ConversionValidation::new(report.valid, model_ready, warnings, errors);

    BioEntity {
        id: structure_id(record),
        entity_type: BioEntityType::Structure,
        source: ConversionSource::new(record.format, None),
        record: ConversionRecord::Structure(Box::new(ConvertedStructureRecord {
            record: record.clone(),
            sequences: extract_structure_sequences(record),
        })),
        validation,
    }
}

/// Convert one parsed molecule record into a unified molecule entity.
pub fn molecule_record_to_bio_entity(record: &MoleculeRecord) -> BioEntity {
    let report = validate_molecule_records(std::slice::from_ref(record));
    let record_report = &report.record_reports[0];
    let warnings = record_report
        .warnings
        .iter()
        .map(molecule_warning)
        .collect::<Vec<_>>();
    let errors = record_report
        .errors
        .iter()
        .map(molecule_error)
        .collect::<Vec<_>>();
    let model_ready = record_report.valid && warnings.is_empty() && errors.is_empty();
    let validation = ConversionValidation::new(record_report.valid, model_ready, warnings, errors);

    BioEntity {
        id: molecule_id(record),
        entity_type: BioEntityType::Molecule,
        source: ConversionSource::new(record.format, Some(record.metadata.source.clone())),
        record: ConversionRecord::Molecule(Box::new(ConvertedMoleculeRecord {
            record: record.clone(),
            format_record: record.to_format_record(),
            derived: derive_molecule_features(record),
        })),
        validation,
    }
}

fn structure_id(record: &StructureRecord) -> String {
    record
        .id
        .clone()
        .or_else(|| record.metadata.title.clone())
        .unwrap_or_else(|| "structure-1".to_string())
}

fn molecule_id(record: &MoleculeRecord) -> String {
    record.id.clone().unwrap_or_else(|| {
        format!(
            "record-{}",
            record.metadata.source.record_index.saturating_add(1)
        )
    })
}

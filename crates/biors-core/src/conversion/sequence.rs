use crate::formats::{fastq_quality_symbol_count, validate_fastq_quality, BioFormat, FastqRecord};
use crate::sequence::{
    detect_sequence_kind_with_metadata, validate_sequence_record, ProteinSequence, SequenceKind,
    SequenceKindDetection, SequenceKindSelection, SequenceRecord,
};

use super::entity::export_bio_entities;
use super::issue::{
    empty_sequence_issue, fastq_quality_issue, fastq_quality_length_mismatch, sequence_validation,
};
use super::types::{
    BioEntity, BioEntityJsonExport, BioEntityType, ConversionRecord, ConversionSource,
    ConvertedSequenceRecord,
};

/// Convert parsed FASTA records into unified sequence entities.
pub fn convert_fasta_records(
    records: &[ProteinSequence],
    kind_selection: SequenceKindSelection,
) -> BioEntityJsonExport {
    export_bio_entities(
        records
            .iter()
            .map(|record| fasta_record_to_bio_entity(record, kind_selection))
            .collect(),
    )
}

/// Convert one parsed FASTA record into a unified sequence entity.
pub fn fasta_record_to_bio_entity(
    record: &ProteinSequence,
    kind_selection: SequenceKindSelection,
) -> BioEntity {
    let sequence = String::from_utf8_lossy(&record.sequence).into_owned();
    let (kind, auto_detection) = selected_kind(&sequence, kind_selection);
    sequence_to_bio_entity(
        record.id.clone(),
        sequence,
        kind,
        auto_detection,
        None,
        None,
        ConversionSource::new(BioFormat::Fasta, None),
    )
}

/// Convert parsed FASTQ records into unified DNA sequence entities.
pub fn convert_fastq_records(records: &[FastqRecord]) -> BioEntityJsonExport {
    export_bio_entities(
        records
            .iter()
            .map(fastq_record_to_bio_entity)
            .collect::<Vec<_>>(),
    )
}

/// Convert one parsed FASTQ record into a unified DNA sequence entity.
pub fn fastq_record_to_bio_entity(record: &FastqRecord) -> BioEntity {
    sequence_to_bio_entity(
        record.id.clone(),
        record.sequence.clone(),
        SequenceKind::Dna,
        None,
        record.description.clone(),
        Some(record.quality.clone()),
        ConversionSource::new(BioFormat::Fastq, Some(record.metadata.clone())),
    )
}

fn selected_kind(
    sequence: &str,
    kind_selection: SequenceKindSelection,
) -> (SequenceKind, Option<SequenceKindDetection>) {
    match kind_selection {
        SequenceKindSelection::Auto => {
            let detection = detect_sequence_kind_with_metadata(sequence);
            (detection.selected_kind, Some(detection))
        }
        SequenceKindSelection::Explicit(kind) => (kind, None),
    }
}

fn sequence_to_bio_entity(
    id: String,
    sequence: String,
    kind: SequenceKind,
    auto_detection: Option<SequenceKindDetection>,
    description: Option<String>,
    quality: Option<String>,
    source: ConversionSource,
) -> BioEntity {
    let record = SequenceRecord::new(id, sequence, kind);
    let validation = validate_sequence_record(&record);
    let sequence_record = SequenceRecord {
        id: validation.id.clone(),
        sequence: validation.sequence.clone(),
        kind: validation.kind,
    };

    let mut extra_errors = Vec::new();
    if sequence_record.sequence.is_empty() {
        extra_errors.push(empty_sequence_issue(&sequence_record.id));
    }
    if let Some(quality) = &quality {
        let sequence_len = sequence_record.sequence.chars().count();
        let quality_len = fastq_quality_symbol_count(quality);
        if sequence_len != quality_len {
            extra_errors.push(fastq_quality_length_mismatch(
                &sequence_record.id,
                sequence_len,
                quality_len,
            ));
        }
        let mut quality_errors = Vec::new();
        validate_fastq_quality(quality, &mut quality_errors);
        extra_errors.extend(quality_errors.iter().map(fastq_quality_issue));
    }

    let validation = sequence_validation(&validation, extra_errors);
    BioEntity {
        id: sequence_record.id.clone(),
        entity_type: BioEntityType::Sequence,
        source,
        record: ConversionRecord::Sequence(ConvertedSequenceRecord {
            sequence: sequence_record,
            description,
            quality,
            auto_detection,
        }),
        validation,
    }
}

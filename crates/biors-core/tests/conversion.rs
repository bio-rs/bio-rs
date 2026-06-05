use biors_core::conversion::{
    convert_fasta_records, convert_fastq_records, export_bio_entities,
    molecule_record_to_bio_entity, structure_record_to_bio_entity, BioEntityType,
    ConversionIssueCode, ConversionRecord, CONVERSION_SCHEMA_VERSION,
};
use biors_core::formats::{parse_fastq_records, BioFormat, FormatMetadata};
use biors_core::molecule::parse_smiles_records;
use biors_core::sequence::{SequenceKind, SequenceKindSelection};
use biors_core::structure::{
    Atom, Chain, Coordinate, MissingResidue, Residue3D, StructureMetadata, StructureRecord,
};

#[test]
fn fasta_records_convert_to_kind_aware_sequence_entities() {
    let records =
        biors_core::parse_fasta_records(">dna\nacgt\n>protein\nMEEPQSDPSV\n").expect("valid FASTA");

    let export = convert_fasta_records(&records, SequenceKindSelection::Auto);

    assert_eq!(export.schema_version, CONVERSION_SCHEMA_VERSION);
    assert_eq!(export.records, 2);
    assert_eq!(export.valid_records, 2);
    assert_eq!(export.model_ready_records, 2);
    assert_eq!(export.entities[0].entity_type, BioEntityType::Sequence);
    assert_eq!(export.entities[0].source.format, BioFormat::Fasta);
    assert!(export.entities[0].source.metadata.is_none());

    let ConversionRecord::Sequence(sequence) = &export.entities[0].record else {
        panic!("expected sequence conversion record");
    };
    assert_eq!(sequence.sequence.id, "dna");
    assert_eq!(sequence.sequence.sequence, "ACGT");
    assert_eq!(sequence.sequence.kind, SequenceKind::Dna);
    assert_eq!(
        sequence
            .auto_detection
            .as_ref()
            .expect("auto detection")
            .selected_kind,
        SequenceKind::Dna
    );
}

#[test]
fn fastq_records_convert_to_dna_entities_with_quality_and_warnings() {
    let records = parse_fastq_records("@read1 sample\nacgtn\n+\n!!!!!\n").expect("valid FASTQ");

    let export = convert_fastq_records(&records);

    assert_eq!(export.records, 1);
    assert_eq!(export.valid_records, 0);
    assert_eq!(export.model_ready_records, 0);
    assert_eq!(export.warning_count, 1);
    assert_eq!(export.entities[0].source.format, BioFormat::Fastq);
    assert_eq!(
        export.entities[0].source.metadata,
        Some(FormatMetadata::new(0, 1, 4))
    );
    assert_eq!(
        export.entities[0].validation.warnings[0].code,
        ConversionIssueCode::SequenceAmbiguousSymbol
    );

    let ConversionRecord::Sequence(sequence) = &export.entities[0].record else {
        panic!("expected sequence conversion record");
    };
    assert_eq!(sequence.sequence.id, "read1");
    assert_eq!(sequence.sequence.kind, SequenceKind::Dna);
    assert_eq!(sequence.sequence.sequence, "ACGTN");
    assert_eq!(sequence.quality.as_deref(), Some("!!!!!"));
    assert_eq!(sequence.description.as_deref(), Some("sample"));
}

#[test]
fn fastq_conversion_rejects_non_ascii_quality_symbols() {
    let records = parse_fastq_records("@read1\nAC\n+\né!\n").expect("parse FASTQ");

    let export = convert_fastq_records(&records);

    assert_eq!(export.records, 1);
    assert_eq!(export.valid_records, 0);
    assert_eq!(export.model_ready_records, 0);
    assert_eq!(export.error_count, 1);
    assert_eq!(
        export.entities[0].validation.errors[0].code,
        ConversionIssueCode::FastqInvalidQualityCharacter
    );
}

#[test]
fn structure_and_molecule_records_share_bioentity_export_shape() {
    let structure = minimal_structure_record();
    let structure_entity = structure_record_to_bio_entity(&structure);
    let molecule = parse_smiles_records("CCO ethanol\n").expect("valid SMILES");
    let molecule_entity = molecule_record_to_bio_entity(&molecule[0]);

    assert_eq!(structure_entity.entity_type, BioEntityType::Structure);
    assert!(structure_entity.validation.valid);
    assert!(structure_entity.validation.model_ready);
    let ConversionRecord::Structure(structure_record) = &structure_entity.record else {
        panic!("expected structure conversion record");
    };
    assert_eq!(structure_record.record.id.as_deref(), Some("1ABC"));
    assert_eq!(
        structure_record.sequences.chains[0].coordinate_sequence,
        "A"
    );

    assert_eq!(molecule_entity.entity_type, BioEntityType::Molecule);
    assert!(molecule_entity.validation.valid);
    assert!(molecule_entity.validation.model_ready);
    let ConversionRecord::Molecule(molecule_record) = &molecule_entity.record else {
        panic!("expected molecule conversion record");
    };
    assert_eq!(molecule_record.format_record.format, BioFormat::Smiles);
    assert_eq!(molecule_record.derived.heavy_atom_count, 3);

    let export = export_bio_entities(vec![structure_entity, molecule_entity]);
    assert_eq!(export.schema_version, CONVERSION_SCHEMA_VERSION);
    assert_eq!(export.records, 2);
    assert_eq!(export.valid_records, 2);
    assert_eq!(export.model_ready_records, 2);
    assert_eq!(
        serde_json::to_value(&export).expect("serialize export")["schema_version"],
        CONVERSION_SCHEMA_VERSION
    );
}

#[test]
fn structure_conversion_with_missing_residue_is_valid_but_not_model_ready() {
    let mut structure = minimal_structure_record();
    structure.metadata.missing_residue_count = 1;
    structure.chains[0].seqres_sequence = Some("AA".to_string());
    structure.chains[0].missing_residues.push(MissingResidue {
        name: "ALA".to_string(),
        chain_id: "A".to_string(),
        sequence_number: 2,
        insertion_code: None,
    });

    let entity = structure_record_to_bio_entity(&structure);

    assert!(entity.validation.valid);
    assert!(!entity.validation.model_ready);
    assert_eq!(entity.validation.warning_count, 1);
    assert_eq!(
        entity.validation.warnings[0].code,
        ConversionIssueCode::StructureValidationWarning
    );
}

#[test]
fn molecule_conversion_with_aromaticity_warning_is_valid_but_not_model_ready() {
    let molecule = parse_smiles_records("c1ccccc1 benzene\n").expect("valid aromatic SMILES");

    let entity = molecule_record_to_bio_entity(&molecule[0]);

    assert!(entity.validation.valid);
    assert!(!entity.validation.model_ready);
    assert_eq!(entity.validation.warning_count, 1);
    assert_eq!(
        entity.validation.warnings[0].code,
        ConversionIssueCode::MoleculeValidationWarning
    );
}

fn minimal_structure_record() -> StructureRecord {
    StructureRecord {
        format: BioFormat::Pdb,
        id: Some("1ABC".to_string()),
        metadata: StructureMetadata {
            title: Some("minimal alanine".to_string()),
            line_count: 1,
            model_count: 1,
            atom_count: 1,
            hetero_atom_count: 0,
            seqres_chain_count: 1,
            missing_residue_count: 0,
        },
        chains: vec![Chain {
            id: "A".to_string(),
            residues: vec![Residue3D {
                name: "ALA".to_string(),
                sequence_number: 1,
                insertion_code: None,
                hetero: false,
                one_letter_code: Some('A'),
                atoms: vec![Atom {
                    serial: 1,
                    name: "CA".to_string(),
                    alternate_location: None,
                    element: Some("C".to_string()),
                    coordinate: Coordinate {
                        x: 1.0,
                        y: 2.0,
                        z: 3.0,
                    },
                    occupancy: Some(1.0),
                    temperature_factor: Some(10.0),
                }],
            }],
            coordinate_sequence: "A".to_string(),
            seqres_sequence: Some("A".to_string()),
            missing_residues: Vec::new(),
        }],
    }
}

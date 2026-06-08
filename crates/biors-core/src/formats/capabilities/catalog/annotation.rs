use super::super::{FormatCapability, FormatSupportStatus};
use crate::formats::BioFormat;

pub(super) fn capabilities() -> Vec<FormatCapability> {
    vec![
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
            record_contract: "FormatRecord with feature table projection requirements".to_string(),
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
    ]
}

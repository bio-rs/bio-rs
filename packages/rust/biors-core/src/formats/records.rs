use serde::{Deserialize, Serialize};

/// Biological file format family recognized by the shared format layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BioFormat {
    /// FASTA sequence records.
    Fasta,
    /// FASTQ sequencing reads with quality strings.
    Fastq,
    /// GFF3 genomic feature annotations.
    Gff3,
    /// GTF genomic feature annotations.
    Gtf,
    /// BED genomic intervals.
    Bed,
    /// VCF variants.
    Vcf,
    /// GenBank flat files.
    Genbank,
    /// UniProt flat file records.
    UniprotFlat,
    /// Protein Data Bank coordinate files.
    Pdb,
    /// PDBx/mmCIF coordinate files.
    Mmcif,
    /// Comma-separated biological tables.
    Csv,
    /// Tab-separated biological tables.
    Tsv,
    /// SMILES molecular line notation.
    Smiles,
    /// Structure-data files / MDL SDfiles.
    Sdf,
    /// Tripos MOL2 molecular graph files.
    Mol2,
}

impl BioFormat {
    /// Stable lower-case format identifier used in JSON and CLI output.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fasta => "fasta",
            Self::Fastq => "fastq",
            Self::Gff3 => "gff3",
            Self::Gtf => "gtf",
            Self::Bed => "bed",
            Self::Vcf => "vcf",
            Self::Genbank => "genbank",
            Self::UniprotFlat => "uniprot-flat",
            Self::Pdb => "pdb",
            Self::Mmcif => "mmcif",
            Self::Csv => "csv",
            Self::Tsv => "tsv",
            Self::Smiles => "smiles",
            Self::Sdf => "sdf",
            Self::Mol2 => "mol2",
        }
    }

    /// Human-readable format name.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Fasta => "FASTA",
            Self::Fastq => "FASTQ",
            Self::Gff3 => "GFF3",
            Self::Gtf => "GTF",
            Self::Bed => "BED",
            Self::Vcf => "VCF",
            Self::Genbank => "GenBank",
            Self::UniprotFlat => "UniProt flat file",
            Self::Pdb => "PDB",
            Self::Mmcif => "mmCIF",
            Self::Csv => "CSV biological table",
            Self::Tsv => "TSV biological table",
            Self::Smiles => "SMILES",
            Self::Sdf => "SDF",
            Self::Mol2 => "MOL2",
        }
    }
}

/// Common source metadata attached to parsed biological file-format records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormatMetadata {
    /// Zero-based record index in the source stream.
    pub record_index: usize,
    /// One-based source line where the record starts.
    pub line_start: usize,
    /// One-based source line where the record ends.
    pub line_end: usize,
}

impl FormatMetadata {
    /// Construct source metadata for one parsed record.
    pub const fn new(record_index: usize, line_start: usize, line_end: usize) -> Self {
        Self {
            record_index,
            line_start,
            line_end,
        }
    }
}

/// A named field in a generic biological format record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormatField {
    /// Stable field name.
    pub name: String,
    /// Field value after format-level normalization.
    pub value: String,
}

impl FormatField {
    /// Construct a generic format field.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Shared record contract used by format-specific parsers and later converters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormatRecord {
    /// Format family used to parse this record.
    pub format: BioFormat,
    /// Stable record identifier from the source format.
    pub id: String,
    /// Source location metadata.
    pub metadata: FormatMetadata,
    /// Format-normalized fields.
    pub fields: Vec<FormatField>,
}

impl FormatRecord {
    /// Construct a shared format record.
    pub fn new(
        format: BioFormat,
        id: impl Into<String>,
        metadata: FormatMetadata,
        fields: Vec<FormatField>,
    ) -> Self {
        Self {
            format,
            id: id.into(),
            metadata,
            fields,
        }
    }
}

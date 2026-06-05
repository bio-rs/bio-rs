use biors_core::tokenizer::ProteinTokenizerProfile;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum TokenizerProfileArg {
    #[default]
    #[value(name = "protein-20")]
    Protein20,
    #[value(name = "protein-20-special")]
    Protein20Special,
    #[value(name = "dna-iupac")]
    DnaIupac,
    #[value(name = "dna-iupac-special")]
    DnaIupacSpecial,
    #[value(name = "rna-iupac")]
    RnaIupac,
    #[value(name = "rna-iupac-special")]
    RnaIupacSpecial,
}

impl From<TokenizerProfileArg> for ProteinTokenizerProfile {
    fn from(value: TokenizerProfileArg) -> Self {
        match value {
            TokenizerProfileArg::Protein20 => Self::Protein20,
            TokenizerProfileArg::Protein20Special => Self::Protein20Special,
            TokenizerProfileArg::DnaIupac => Self::DnaIupac,
            TokenizerProfileArg::DnaIupacSpecial => Self::DnaIupacSpecial,
            TokenizerProfileArg::RnaIupac => Self::RnaIupac,
            TokenizerProfileArg::RnaIupacSpecial => Self::RnaIupacSpecial,
        }
    }
}

impl From<ProteinTokenizerProfile> for TokenizerProfileArg {
    fn from(value: ProteinTokenizerProfile) -> Self {
        match value {
            ProteinTokenizerProfile::Protein20 => Self::Protein20,
            ProteinTokenizerProfile::Protein20Special => Self::Protein20Special,
            ProteinTokenizerProfile::DnaIupac => Self::DnaIupac,
            ProteinTokenizerProfile::DnaIupacSpecial => Self::DnaIupacSpecial,
            ProteinTokenizerProfile::RnaIupac => Self::RnaIupac,
            ProteinTokenizerProfile::RnaIupacSpecial => Self::RnaIupacSpecial,
        }
    }
}

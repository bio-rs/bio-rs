use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};

use crate::formats::BioFormat;
use crate::structure::residue_codes::protein_code_for_residue;
use crate::structure::{
    Atom, Chain, MissingResidue, Residue3D, StructureMetadata, StructureRecord,
};

#[derive(Default)]
pub(super) struct StructureBuilder {
    pub(super) pdb_id: Option<String>,
    pub(super) title_lines: Vec<String>,
    pub(super) line_count: usize,
    pub(super) model_count: usize,
    pub(super) atom_count: usize,
    pub(super) hetero_atom_count: usize,
    pub(super) seqres: BTreeMap<String, Vec<String>>,
    pub(super) missing_residues: Vec<MissingResidue>,
    chains: Vec<ChainBuilder>,
    chain_index: HashMap<String, usize>,
}

impl StructureBuilder {
    pub(super) fn add_atom(&mut self, row: ParsedAtom) {
        self.atom_count += usize::from(!row.hetero);
        self.hetero_atom_count += usize::from(row.hetero);
        let index = *self
            .chain_index
            .entry(row.chain_id.clone())
            .or_insert_with(|| {
                self.chains.push(ChainBuilder::new(row.chain_id.clone()));
                self.chains.len() - 1
            });
        self.chains[index].add_atom(row);
    }

    pub(super) fn finish(self) -> StructureRecord {
        let title = join_title_lines(&self.title_lines);
        let missing_by_chain = missing_residues_by_chain(&self.missing_residues);
        let chains = self
            .chains
            .into_iter()
            .map(|chain| chain.finish(&self.seqres, &missing_by_chain))
            .collect();
        StructureRecord {
            format: BioFormat::Pdb,
            id: self.pdb_id,
            metadata: StructureMetadata {
                title,
                line_count: self.line_count,
                model_count: self.model_count,
                atom_count: self.atom_count,
                hetero_atom_count: self.hetero_atom_count,
                seqres_chain_count: self.seqres.len(),
                missing_residue_count: self.missing_residues.len(),
            },
            chains,
        }
    }
}

struct ChainBuilder {
    id: String,
    residues: Vec<Residue3D>,
    residue_index: HashMap<ResidueKey, usize>,
}

impl ChainBuilder {
    fn new(id: String) -> Self {
        Self {
            id,
            residues: Vec::new(),
            residue_index: HashMap::new(),
        }
    }

    fn add_atom(&mut self, row: ParsedAtom) {
        let key = ResidueKey::from(&row);
        let index = *self.residue_index.entry(key).or_insert_with(|| {
            let one_letter_code = if row.hetero {
                None
            } else {
                Some(protein_code_for_residue(&row.residue_name).unwrap_or('X'))
            };
            self.residues.push(Residue3D {
                name: row.residue_name.clone(),
                sequence_number: row.sequence_number,
                insertion_code: row.insertion_code,
                hetero: row.hetero,
                one_letter_code,
                atoms: Vec::new(),
            });
            self.residues.len() - 1
        });
        self.residues[index].atoms.push(row.atom);
    }

    fn finish(
        self,
        seqres: &BTreeMap<String, Vec<String>>,
        missing_by_chain: &BTreeMap<String, Vec<MissingResidue>>,
    ) -> Chain {
        let coordinate_sequence = coordinate_sequence(&self.residues);
        let seqres_sequence = seqres.get(&self.id).map(|names| residue_sequence(names));
        Chain {
            id: self.id.clone(),
            residues: self.residues,
            coordinate_sequence,
            seqres_sequence,
            missing_residues: missing_by_chain.get(&self.id).cloned().unwrap_or_default(),
        }
    }
}

pub(super) struct ParsedAtom {
    pub(super) chain_id: String,
    pub(super) residue_name: String,
    pub(super) sequence_number: i32,
    pub(super) insertion_code: Option<char>,
    pub(super) hetero: bool,
    pub(super) atom: Atom,
}

#[derive(Clone, Eq)]
struct ResidueKey {
    name: String,
    sequence_number: i32,
    insertion_code: Option<char>,
    hetero: bool,
}

impl From<&ParsedAtom> for ResidueKey {
    fn from(row: &ParsedAtom) -> Self {
        Self {
            name: row.residue_name.clone(),
            sequence_number: row.sequence_number,
            insertion_code: row.insertion_code,
            hetero: row.hetero,
        }
    }
}

impl PartialEq for ResidueKey {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.sequence_number == other.sequence_number
            && self.insertion_code == other.insertion_code
            && self.hetero == other.hetero
    }
}

impl Hash for ResidueKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.sequence_number.hash(state);
        self.insertion_code.hash(state);
        self.hetero.hash(state);
    }
}

fn join_title_lines(title_lines: &[String]) -> Option<String> {
    (!title_lines.is_empty()).then(|| title_lines.join(" "))
}

fn residue_sequence(names: &[String]) -> String {
    names
        .iter()
        .map(|name| protein_code_for_residue(name).unwrap_or('X'))
        .collect()
}

fn coordinate_sequence(residues: &[Residue3D]) -> String {
    residues
        .iter()
        .filter(|residue| !residue.hetero)
        .filter_map(|residue| residue.one_letter_code)
        .collect()
}

fn missing_residues_by_chain(residues: &[MissingResidue]) -> BTreeMap<String, Vec<MissingResidue>> {
    let mut grouped: BTreeMap<String, Vec<MissingResidue>> = BTreeMap::new();
    for residue in residues {
        grouped
            .entry(residue.chain_id.clone())
            .or_default()
            .push(residue.clone());
    }
    grouped
}

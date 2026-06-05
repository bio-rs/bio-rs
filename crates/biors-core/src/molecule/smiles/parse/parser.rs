use std::collections::BTreeMap;

use crate::formats::{BioFormat, FormatMetadata};
use crate::molecule::graph::disconnected_components;
use crate::molecule::{
    AtomGraph, BondGraph, BondOrder, MolecularGraph, MoleculeAtom, MoleculeBond, MoleculeMetadata,
    MoleculeRecord,
};

use super::syntax::{bond_order_from_symbol, bond_stereochemistry_from_symbol, parse_percent_ring};
use crate::molecule::smiles::SmilesParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingRing {
    pub atom_index: usize,
    pub order: Option<BondOrder>,
    pub stereochemistry: Option<String>,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingBond {
    pub order: Option<BondOrder>,
    pub stereochemistry: Option<String>,
    pub column: usize,
}

pub(super) struct SmilesParser<'a> {
    pub(super) smiles: &'a str,
    pub(super) id: Option<String>,
    pub(super) line: usize,
    pub(super) record_index: usize,
    pub(super) atoms: Vec<MoleculeAtom>,
    pub(super) bonds: Vec<MoleculeBond>,
    pub(super) branch_stack: Vec<(usize, usize)>,
    pub(super) rings: BTreeMap<u16, PendingRing>,
    pub(super) current_atom: Option<usize>,
    pub(super) pending_bond: Option<PendingBond>,
    pub(super) branch_count: usize,
    pub(super) ring_closure_count: usize,
}

impl<'a> SmilesParser<'a> {
    pub(super) fn new(
        smiles: &'a str,
        id: Option<String>,
        line: usize,
        record_index: usize,
    ) -> Self {
        Self {
            smiles,
            id,
            line,
            record_index,
            atoms: Vec::new(),
            bonds: Vec::new(),
            branch_stack: Vec::new(),
            rings: BTreeMap::new(),
            current_atom: None,
            pending_bond: None,
            branch_count: 0,
            ring_closure_count: 0,
        }
    }

    pub(super) fn parse(mut self) -> Result<MoleculeRecord, SmilesParseError> {
        let chars: Vec<_> = self.smiles.char_indices().collect();
        let mut position = 0usize;
        while position < chars.len() {
            let (byte_index, symbol) = chars[position];
            let column = byte_index + 1;
            match symbol {
                '(' => {
                    let Some(atom_index) = self.current_atom else {
                        return Err(self.invalid_branch(column));
                    };
                    self.branch_count += 1;
                    self.branch_stack.push((atom_index, column));
                    position += 1;
                }
                ')' => {
                    let Some((atom_index, _)) = self.branch_stack.pop() else {
                        return Err(self.unmatched_branch(column));
                    };
                    self.current_atom = Some(atom_index);
                    self.pending_bond = None;
                    position += 1;
                }
                '-' | '=' | '#' | '$' | ':' | '/' | '\\' => {
                    if self.pending_bond.is_some() || self.current_atom.is_none() {
                        return Err(self.dangling_bond(column));
                    }
                    self.pending_bond = Some(PendingBond {
                        order: bond_order_from_symbol(symbol),
                        stereochemistry: bond_stereochemistry_from_symbol(symbol),
                        column,
                    });
                    position += 1;
                }
                '.' => {
                    if self.pending_bond.is_some() {
                        return Err(self.dangling_bond(column));
                    }
                    self.current_atom = None;
                    position += 1;
                }
                '0'..='9' => {
                    self.consume_ring(symbol.to_digit(10).expect("digit") as u16, column)?;
                    position += 1;
                }
                '%' => {
                    let (ring, next_position) = parse_percent_ring(&chars, position)
                        .ok_or_else(|| self.unexpected_character(column, symbol))?;
                    self.consume_ring(ring, column)?;
                    position = next_position;
                }
                '[' => {
                    let (atom, next_position) = self.parse_bracket_atom(&chars, position)?;
                    self.push_atom(atom);
                    position = next_position;
                }
                '*' => {
                    let atom = self.atom("*", "*", false, false, None, 0, 0, None, None);
                    self.push_atom(atom);
                    position += 1;
                }
                _ => {
                    if let Some((atom, next_position)) = self.parse_organic_atom(&chars, position) {
                        self.push_atom(atom);
                        position = next_position;
                    } else {
                        return Err(self.unexpected_character(column, symbol));
                    }
                }
            }
        }

        if let Some(pending) = &self.pending_bond {
            return Err(self.dangling_bond(pending.column));
        }
        if let Some((_, column)) = self.branch_stack.last() {
            return Err(self.unclosed_branch(*column));
        }
        if let Some((ring, pending)) = self.rings.iter().next() {
            return Err(self.unclosed_ring(*ring, pending.column));
        }

        let metadata = MoleculeMetadata {
            source: FormatMetadata::new(self.record_index, self.line, self.line),
            atom_count: self.atoms.len(),
            bond_count: self.bonds.len(),
            branch_count: self.branch_count,
            ring_closure_count: self.ring_closure_count,
            disconnected_component_count: disconnected_components(self.atoms.len(), &self.bonds),
            aromatic_atom_count: self.atoms.iter().filter(|atom| atom.aromatic).count(),
        };
        Ok(MoleculeRecord {
            format: BioFormat::Smiles,
            id: self.id,
            source: self.smiles.to_string(),
            metadata,
            graph: MolecularGraph {
                atoms: AtomGraph { atoms: self.atoms },
                bonds: BondGraph { bonds: self.bonds },
            },
            properties: Vec::new(),
        })
    }

    fn push_atom(&mut self, atom: MoleculeAtom) {
        let atom_index = atom.index;
        self.atoms.push(atom);
        if let Some(source_atom) = self.current_atom {
            let pending = self.pending_bond.take();
            self.add_bond(source_atom, atom_index, pending, false);
        }
        self.current_atom = Some(atom_index);
    }

    fn add_bond(
        &mut self,
        source_atom: usize,
        target_atom: usize,
        pending: Option<PendingBond>,
        ring_closure: bool,
    ) {
        let order = pending
            .as_ref()
            .and_then(|bond| bond.order)
            .unwrap_or_else(|| self.inferred_bond_order(source_atom, target_atom));
        let stereochemistry = pending.and_then(|bond| bond.stereochemistry);
        self.bonds.push(MoleculeBond {
            index: self.bonds.len(),
            source_atom,
            target_atom,
            order,
            ring_closure,
            stereochemistry,
        });
    }

    fn inferred_bond_order(&self, source_atom: usize, target_atom: usize) -> BondOrder {
        let source = &self.atoms[source_atom];
        let target = &self.atoms[target_atom];
        if source.aromatic && target.aromatic {
            BondOrder::Aromatic
        } else {
            BondOrder::Single
        }
    }

    fn consume_ring(&mut self, ring: u16, column: usize) -> Result<(), SmilesParseError> {
        let Some(atom_index) = self.current_atom else {
            return Err(self.invalid_ring_closure(column));
        };
        let pending = self.pending_bond.take();
        if let Some(open) = self.rings.remove(&ring) {
            let order = pending
                .as_ref()
                .and_then(|bond| bond.order)
                .or(open.order)
                .unwrap_or_else(|| self.inferred_bond_order(open.atom_index, atom_index));
            let stereochemistry = pending
                .as_ref()
                .and_then(|bond| bond.stereochemistry.clone())
                .or(open.stereochemistry);
            self.bonds.push(MoleculeBond {
                index: self.bonds.len(),
                source_atom: open.atom_index,
                target_atom: atom_index,
                order,
                ring_closure: true,
                stereochemistry,
            });
            self.ring_closure_count += 1;
        } else {
            self.rings.insert(
                ring,
                PendingRing {
                    atom_index,
                    order: pending.as_ref().and_then(|bond| bond.order),
                    stereochemistry: pending.and_then(|bond| bond.stereochemistry),
                    column,
                },
            );
        }
        Ok(())
    }
}

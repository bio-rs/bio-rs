use std::collections::BTreeMap;

use super::hash::stable_hash_hex;
use crate::molecule::{MoleculeBond, MoleculeRecord};

pub(super) fn canonical_graph_key(record: &MoleculeRecord) -> String {
    let mut atom_labels: Vec<_> = record
        .graph
        .atoms
        .atoms
        .iter()
        .map(|atom| {
            (
                atom.index,
                format!(
                    "{}:{}:{}:{}:{}",
                    atom.element,
                    atom.charge,
                    atom.explicit_hydrogens,
                    usize::from(atom.aromatic),
                    degree(atom.index, &record.graph.bonds.bonds)
                ),
            )
        })
        .collect();
    for _ in 0..4 {
        let previous: BTreeMap<_, _> = atom_labels
            .iter()
            .map(|(index, label)| (*index, label.clone()))
            .collect();
        atom_labels = record
            .graph
            .atoms
            .atoms
            .iter()
            .map(|atom| {
                let mut neighbors: Vec<_> = record
                    .graph
                    .bonds
                    .bonds
                    .iter()
                    .filter_map(|bond| {
                        let other = if bond.source_atom == atom.index {
                            Some(bond.target_atom)
                        } else if bond.target_atom == atom.index {
                            Some(bond.source_atom)
                        } else {
                            None
                        }?;
                        Some(format!(
                            "{}:{}",
                            bond.order.as_str(),
                            previous.get(&other).expect("neighbor label")
                        ))
                    })
                    .collect();
                neighbors.sort();
                (
                    atom.index,
                    stable_hash_hex(&format!(
                        "{}|{}",
                        previous.get(&atom.index).expect("atom label"),
                        neighbors.join(",")
                    )),
                )
            })
            .collect();
    }

    let labels: BTreeMap<_, _> = atom_labels.into_iter().collect();
    let mut atoms: Vec<_> = record
        .graph
        .atoms
        .atoms
        .iter()
        .map(|atom| {
            format!(
                "{}:{}:{}:{}",
                labels.get(&atom.index).expect("atom label"),
                atom.element,
                atom.charge,
                atom.explicit_hydrogens
            )
        })
        .collect();
    atoms.sort();
    let mut bonds: Vec<_> = record
        .graph
        .bonds
        .bonds
        .iter()
        .map(|bond| {
            let left = labels.get(&bond.source_atom).expect("left label");
            let right = labels.get(&bond.target_atom).expect("right label");
            if left <= right {
                format!("{}-{}-{}", left, bond.order.as_str(), right)
            } else {
                format!("{}-{}-{}", right, bond.order.as_str(), left)
            }
        })
        .collect();
    bonds.sort();
    format!(
        "biors-graph-v0;atoms={};bonds={}",
        atoms.join(";"),
        bonds.join(";")
    )
}

fn incident_bonds(
    atom_index: usize,
    bonds: &[MoleculeBond],
) -> impl Iterator<Item = &MoleculeBond> {
    bonds
        .iter()
        .filter(move |bond| bond.source_atom == atom_index || bond.target_atom == atom_index)
}

fn degree(atom_index: usize, bonds: &[MoleculeBond]) -> usize {
    incident_bonds(atom_index, bonds).count()
}

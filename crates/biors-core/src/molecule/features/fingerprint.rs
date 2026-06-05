use std::collections::{BTreeMap, BTreeSet};

use super::hash::hash_to_bit;
use super::FINGERPRINT_BITS;
use crate::molecule::{MoleculeAtom, MoleculeBond, MoleculeFingerprint, MoleculeRecord};

pub(super) fn fingerprint(record: &MoleculeRecord) -> MoleculeFingerprint {
    let mut set_bits = BTreeSet::new();
    let atom_labels: BTreeMap<_, _> = record
        .graph
        .atoms
        .atoms
        .iter()
        .map(|atom| (atom.index, atom_feature(atom)))
        .collect();
    for atom in &record.graph.atoms.atoms {
        set_bits.insert(hash_to_bit(&atom_feature(atom), FINGERPRINT_BITS));
        let mut frontier = vec![atom.index];
        let mut seen = BTreeSet::from([atom.index]);
        for radius in 1..=2 {
            let mut next = Vec::new();
            let mut neighborhood = Vec::new();
            for current in &frontier {
                for bond in incident_bonds(*current, &record.graph.bonds.bonds) {
                    let other = if bond.source_atom == *current {
                        bond.target_atom
                    } else {
                        bond.source_atom
                    };
                    neighborhood.push(format!(
                        "{}:{}",
                        bond.order.as_str(),
                        atom_labels.get(&other).expect("atom label")
                    ));
                    if seen.insert(other) {
                        next.push(other);
                    }
                }
            }
            neighborhood.sort();
            set_bits.insert(hash_to_bit(
                &format!(
                    "r{radius}:{}:{}",
                    atom_feature(atom),
                    neighborhood.join(",")
                ),
                FINGERPRINT_BITS,
            ));
            frontier = next;
        }
    }
    MoleculeFingerprint {
        algorithm: "biors-ecfp-lite-v0".to_string(),
        bits: FINGERPRINT_BITS,
        set_bits: set_bits.into_iter().collect(),
    }
}

fn atom_feature(atom: &MoleculeAtom) -> String {
    format!(
        "{}:{}:{}:{}",
        atom.element,
        atom.charge,
        atom.explicit_hydrogens,
        usize::from(atom.aromatic)
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

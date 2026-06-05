use std::collections::BTreeMap;

mod fingerprint;
mod graph_key;
mod hash;

use fingerprint::fingerprint;
use graph_key::canonical_graph_key;

use super::types::{BondOrder, MoleculeBond, MoleculeDerivedFeatures, MoleculeRecord};

const FINGERPRINT_BITS: usize = 2048;

/// Derive deterministic graph features from a parsed molecule.
pub fn derive_molecule_features(record: &MoleculeRecord) -> MoleculeDerivedFeatures {
    let formula_counts = formula_counts(record);
    MoleculeDerivedFeatures {
        canonical_graph_key: canonical_graph_key(record),
        formula: formula_string(&formula_counts),
        exact_mass: exact_mass(&formula_counts),
        heavy_atom_count: record
            .graph
            .atoms
            .atoms
            .iter()
            .filter(|atom| atom.element != "H" && atom.element != "*")
            .count(),
        hetero_atom_count: record
            .graph
            .atoms
            .atoms
            .iter()
            .filter(|atom| !matches!(atom.element.as_str(), "C" | "H" | "*"))
            .count(),
        ring_bond_count: record
            .graph
            .bonds
            .bonds
            .iter()
            .filter(|bond| bond.ring_closure)
            .count(),
        rotatable_bond_count: rotatable_bond_count(record),
        hydrogen_bond_donor_count: hydrogen_bond_donor_count(record),
        hydrogen_bond_acceptor_count: hydrogen_bond_acceptor_count(record),
        formal_charge: record
            .graph
            .atoms
            .atoms
            .iter()
            .map(|atom| i32::from(atom.charge))
            .sum(),
        fingerprint: fingerprint(record),
    }
}

fn formula_counts(record: &MoleculeRecord) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for atom in &record.graph.atoms.atoms {
        if atom.element != "*" {
            *counts.entry(atom.element.clone()).or_default() += 1;
        }
        if atom.explicit_hydrogens > 0 {
            *counts.entry("H".to_string()).or_default() += usize::from(atom.explicit_hydrogens);
        }
    }
    counts
}

fn formula_string(counts: &BTreeMap<String, usize>) -> String {
    let mut parts = Vec::new();
    for element in ["C", "H"] {
        if let Some(count) = counts.get(element) {
            parts.push(format_element_count(element, *count));
        }
    }
    for (element, count) in counts {
        if element != "C" && element != "H" {
            parts.push(format_element_count(element, *count));
        }
    }
    parts.join("")
}

fn format_element_count(element: &str, count: usize) -> String {
    if count == 1 {
        element.to_string()
    } else {
        format!("{element}{count}")
    }
}

fn exact_mass(counts: &BTreeMap<String, usize>) -> f64 {
    counts
        .iter()
        .map(|(element, count)| atomic_mass(element).unwrap_or(0.0) * *count as f64)
        .sum()
}

fn atomic_mass(element: &str) -> Option<f64> {
    Some(match element {
        "H" => 1.007_825,
        "B" => 11.009_305,
        "C" => 12.0,
        "N" => 14.003_074,
        "O" => 15.994_915,
        "F" => 18.998_403,
        "P" => 30.973_762,
        "S" => 31.972_071,
        "Cl" => 34.968_853,
        "Br" => 78.918_338,
        "I" => 126.904_468,
        "Si" => 27.976_927,
        "Se" => 79.916_522,
        "As" => 74.921_596,
        _ => return None,
    })
}

fn rotatable_bond_count(record: &MoleculeRecord) -> usize {
    record
        .graph
        .bonds
        .bonds
        .iter()
        .filter(|bond| {
            bond.order == BondOrder::Single
                && !bond.ring_closure
                && degree(bond.source_atom, &record.graph.bonds.bonds) > 1
                && degree(bond.target_atom, &record.graph.bonds.bonds) > 1
        })
        .count()
}

fn hydrogen_bond_donor_count(record: &MoleculeRecord) -> usize {
    record
        .graph
        .atoms
        .atoms
        .iter()
        .filter(|atom| {
            matches!(atom.element.as_str(), "N" | "O" | "S") && atom.explicit_hydrogens > 0
        })
        .count()
}

fn hydrogen_bond_acceptor_count(record: &MoleculeRecord) -> usize {
    record
        .graph
        .atoms
        .atoms
        .iter()
        .filter(|atom| matches!(atom.element.as_str(), "N" | "O" | "S") && atom.charge <= 0)
        .count()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecule::parse_smiles_records;

    #[test]
    fn graph_key_is_stable_for_reversed_ethanol() {
        let left = parse_smiles_records("CCO\n").expect("parse left");
        let right = parse_smiles_records("OCC\n").expect("parse right");

        assert_eq!(
            derive_molecule_features(&left[0]).canonical_graph_key,
            derive_molecule_features(&right[0]).canonical_graph_key
        );
    }

    #[test]
    fn descriptors_count_explicit_hydrogens() {
        let records = parse_smiles_records("[NH4+]\n").expect("parse ammonium");
        let features = derive_molecule_features(&records[0]);

        assert_eq!(features.formula, "H4N");
        assert_eq!(features.formal_charge, 1);
        assert_eq!(features.hydrogen_bond_donor_count, 1);
        assert!(!features.fingerprint.set_bits.is_empty());
    }
}

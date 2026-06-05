use super::types::MoleculeBond;

pub(crate) fn disconnected_components(atom_count: usize, bonds: &[MoleculeBond]) -> usize {
    if atom_count == 0 {
        return 0;
    }
    let mut parent: Vec<_> = (0..atom_count).collect();
    for bond in bonds {
        union(&mut parent, bond.source_atom, bond.target_atom);
    }
    (0..atom_count)
        .filter(|atom| find(&mut parent, *atom) == *atom)
        .count()
}

fn union(parent: &mut [usize], left: usize, right: usize) {
    let left_root = find(parent, left);
    let right_root = find(parent, right);
    if left_root != right_root {
        parent[right_root] = left_root;
    }
}

fn find(parent: &mut [usize], node: usize) -> usize {
    if parent[node] != node {
        parent[node] = find(parent, parent[node]);
    }
    parent[node]
}

use super::super::types::{ProteinStructureMapping, ProteinStructureMappingStatus};

pub(super) fn map_coordinate_to_seqres(
    coordinate_sequence: &str,
    seqres_sequence: &Option<String>,
) -> ProteinStructureMapping {
    let Some(seqres) = seqres_sequence else {
        return ProteinStructureMapping {
            status: ProteinStructureMappingStatus::MissingSeqres,
            message: "chain has no SEQRES sequence to map against".to_string(),
            coordinate_to_seqres_positions: vec![None; coordinate_sequence.chars().count()],
        };
    };
    if coordinate_sequence == seqres {
        return ProteinStructureMapping {
            status: ProteinStructureMappingStatus::Exact,
            message: "coordinate sequence exactly matches SEQRES".to_string(),
            coordinate_to_seqres_positions: (1..=coordinate_sequence.chars().count())
                .map(Some)
                .collect(),
        };
    }
    match subsequence_positions(coordinate_sequence, seqres) {
        Some(positions) => ProteinStructureMapping {
            status: ProteinStructureMappingStatus::CoordinateSubsequence,
            message: "coordinate sequence is an ordered subsequence of SEQRES".to_string(),
            coordinate_to_seqres_positions: positions.into_iter().map(Some).collect(),
        },
        None => ProteinStructureMapping {
            status: ProteinStructureMappingStatus::Mismatch,
            message: "coordinate sequence cannot be mapped as an ordered subsequence of SEQRES"
                .to_string(),
            coordinate_to_seqres_positions: vec![None; coordinate_sequence.chars().count()],
        },
    }
}

fn subsequence_positions(query: &str, target: &str) -> Option<Vec<usize>> {
    let mut target_iter = target.chars().enumerate();
    let mut positions = Vec::new();
    for query_char in query.chars() {
        let (target_index, _) = target_iter.find(|(_, target_char)| *target_char == query_char)?;
        positions.push(target_index + 1);
    }
    Some(positions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_mapping_reports_ordered_subsequence_positions() {
        let mapping = map_coordinate_to_seqres("AE", &Some("ACDE".to_string()));

        assert_eq!(
            mapping.status,
            ProteinStructureMappingStatus::CoordinateSubsequence
        );
        assert_eq!(
            mapping.coordinate_to_seqres_positions,
            vec![Some(1), Some(4)]
        );
    }

    #[test]
    fn sequence_mapping_reports_mismatch_when_order_breaks() {
        let mapping = map_coordinate_to_seqres("EA", &Some("ACDE".to_string()));

        assert_eq!(mapping.status, ProteinStructureMappingStatus::Mismatch);
        assert_eq!(mapping.coordinate_to_seqres_positions, vec![None, None]);
    }
}

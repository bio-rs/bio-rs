use super::{ContentMismatchDiff, FirstDifference, OutputDiffReport};
use crate::sha256_digest;

/// Build a canonical diff report for two outputs.
pub fn diff_output_bytes(
    expected_path: &str,
    observed_path: &str,
    expected: &[u8],
    observed: &[u8],
) -> OutputDiffReport {
    let matches = contents_match(expected, observed);
    OutputDiffReport {
        expected_path: expected_path.to_string(),
        observed_path: observed_path.to_string(),
        expected_sha256: sha256_digest(expected),
        observed_sha256: sha256_digest(observed),
        matches,
        content_diff: (!matches)
            .then(|| content_mismatch_diff(expected_path, observed_path, expected, observed)),
    }
}

pub(super) fn contents_match(expected: &[u8], observed: &[u8]) -> bool {
    match (
        serde_json::from_slice::<serde_json::Value>(expected),
        serde_json::from_slice::<serde_json::Value>(observed),
    ) {
        (Ok(expected), Ok(observed)) => expected == observed,
        _ => expected == observed,
    }
}

pub(super) fn content_mismatch_diff(
    expected_path: &str,
    observed_path: &str,
    expected: &[u8],
    observed: &[u8],
) -> ContentMismatchDiff {
    let expected = canonical_content_bytes(expected);
    let observed = canonical_content_bytes(observed);
    ContentMismatchDiff {
        expected_path: expected_path.to_string(),
        observed_path: observed_path.to_string(),
        expected_len: expected.len(),
        observed_len: observed.len(),
        first_difference: first_difference(&expected, &observed),
    }
}

fn canonical_content_bytes(bytes: &[u8]) -> Vec<u8> {
    match serde_json::from_slice::<serde_json::Value>(bytes) {
        Ok(json) => serde_json::to_vec(&json).unwrap_or_else(|_| bytes.to_vec()),
        Err(_) => bytes.to_vec(),
    }
}

fn first_difference(expected: &[u8], observed: &[u8]) -> Option<FirstDifference> {
    let shared_len = expected.len().min(observed.len());
    for index in 0..shared_len {
        if expected[index] != observed[index] {
            return Some(FirstDifference {
                byte_offset: index,
                expected_byte: Some(expected[index]),
                observed_byte: Some(observed[index]),
            });
        }
    }

    if expected.len() != observed.len() {
        return Some(FirstDifference {
            byte_offset: shared_len,
            expected_byte: expected.get(shared_len).copied(),
            observed_byte: observed.get(shared_len).copied(),
        });
    }

    None
}

use super::super::types::VerificationIssueCode;
use crate::hash::sha256_bytes_digest;

pub(super) fn validate_declared_hash(
    path: &str,
    bytes: &[u8],
    declared_hash: Option<&str>,
    code: VerificationIssueCode,
    label: &str,
) -> Option<(VerificationIssueCode, String)> {
    let declared_hash = declared_hash?;
    let actual_hash = sha256_bytes_digest(bytes);
    if actual_hash == declared_hash {
        return None;
    }

    Some((
        code,
        format!("{label} for '{path}': expected '{declared_hash}' but computed '{actual_hash}'"),
    ))
}

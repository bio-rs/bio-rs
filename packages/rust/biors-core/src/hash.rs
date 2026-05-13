use sha2::{Digest, Sha256};
use std::borrow::Cow;

/// Compute a canonical SHA-256 digest string in `sha256:<hex>` form.
pub fn sha256_digest(bytes: &[u8]) -> String {
    let normalized = canonical_hash_bytes(bytes);
    let digest = Sha256::digest(&normalized);
    format!("sha256:{digest:x}")
}

/// Return true when a checksum uses the supported `sha256:<64 hex>` format.
pub fn is_sha256_checksum(checksum: &str) -> bool {
    let Some(hex) = checksum.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn canonical_hash_bytes(bytes: &[u8]) -> Cow<'_, [u8]> {
    match serde_json::from_slice::<serde_json::Value>(bytes) {
        Ok(json) => match serde_json::to_vec(&json) {
            Ok(vec) => Cow::Owned(vec),
            Err(_) => Cow::Borrowed(bytes),
        },
        Err(_) => Cow::Borrowed(bytes),
    }
}

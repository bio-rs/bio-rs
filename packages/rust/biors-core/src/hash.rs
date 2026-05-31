use sha2::{Digest, Sha256};
use std::borrow::Cow;

/// Incrementally compute a raw byte-for-byte SHA-256 digest.
#[derive(Default)]
pub(crate) struct Sha256ByteHasher {
    inner: Sha256,
}

impl Sha256ByteHasher {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn update(&mut self, bytes: &[u8]) {
        self.inner.update(bytes);
    }

    pub(crate) fn finalize(self) -> String {
        let digest = self.inner.finalize();
        format!("sha256:{digest:x}")
    }
}

/// Compute a raw byte-for-byte SHA-256 digest string in `sha256:<hex>` form.
pub fn sha256_bytes_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{digest:x}")
}

/// Compute a canonical JSON SHA-256 digest string in `sha256:<hex>` form.
///
/// If `bytes` are valid JSON, the parsed JSON value is serialized before
/// hashing so semantically equivalent JSON values produce the same digest. For
/// non-JSON input this falls back to a raw byte digest.
pub fn sha256_canonical_json_digest(bytes: &[u8]) -> String {
    let normalized = canonical_hash_bytes(bytes);
    sha256_bytes_digest(&normalized)
}

/// Compute a canonical JSON SHA-256 digest string in `sha256:<hex>` form.
///
/// Retained for public API compatibility. Prefer `sha256_bytes_digest` for
/// artifact/file checksums and `sha256_canonical_json_digest` for semantic JSON
/// comparisons.
pub fn sha256_digest(bytes: &[u8]) -> String {
    sha256_canonical_json_digest(bytes)
}

/// Return true when a checksum uses the supported `sha256:<64 hex>` format.
pub fn is_sha256_checksum(checksum: &str) -> bool {
    let Some(hex) = checksum.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
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

#[cfg(test)]
mod tests {
    use super::{is_sha256_checksum, sha256_bytes_digest, sha256_canonical_json_digest};

    #[test]
    fn byte_digest_preserves_json_whitespace() {
        let compact = br#"{"a":1}"#;
        let spaced = br#"{
          "a": 1
        }"#;

        assert_ne!(sha256_bytes_digest(compact), sha256_bytes_digest(spaced));
        assert_eq!(
            sha256_canonical_json_digest(compact),
            sha256_canonical_json_digest(spaced)
        );
    }

    #[test]
    fn checksum_format_requires_lowercase_hex() {
        assert!(is_sha256_checksum(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ));
        assert!(!is_sha256_checksum(
            "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        ));
        assert!(!is_sha256_checksum(
            "sha256:gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg"
        ));
    }
}

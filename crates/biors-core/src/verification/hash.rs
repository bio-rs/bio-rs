/// Compute the stable FNV-1a input hash used in CLI JSON payloads.
pub fn stable_input_hash(input: &str) -> String {
    let mut hasher = StableInputHasher::new();
    hasher.update(input.as_bytes());
    hasher.finalize()
}

/// Return whether a string matches the stable `fnv1a64:<16 lowercase hex>` form.
pub fn is_stable_input_hash(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("fnv1a64:") else {
        return false;
    };
    hex.len() == 16
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[derive(Debug, Clone, Copy)]
/// Incremental stable input hasher for streaming reader paths.
pub struct StableInputHasher {
    hash: u64,
}

impl StableInputHasher {
    /// Create a new FNV-1a hasher initialized with the standard offset basis.
    pub const fn new() -> Self {
        Self {
            hash: 0xcbf29ce484222325,
        }
    }

    pub fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash ^= u64::from(*byte);
            self.hash = self.hash.wrapping_mul(0x100000001b3);
        }
    }

    /// Return the final hash string in `fnv1a64:<hex>` form.
    pub fn finalize(self) -> String {
        let hash = self.hash;
        format!("fnv1a64:{hash:016x}")
    }
}

impl Default for StableInputHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::is_stable_input_hash;

    #[test]
    fn stable_input_hash_validation_matches_schema_shape() {
        assert!(is_stable_input_hash("fnv1a64:08a331cb13c7bd72"));
        assert!(!is_stable_input_hash("sha256:08a331cb13c7bd72"));
        assert!(!is_stable_input_hash("fnv1a64:08A331CB13C7BD72"));
        assert!(!is_stable_input_hash("fnv1a64:08a331cb13c7bd7"));
        assert!(!is_stable_input_hash("fnv1a64:08a331cb13c7bd7z"));
    }
}

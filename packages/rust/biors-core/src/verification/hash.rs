/// Compute the stable FNV-1a input hash used in CLI JSON payloads.
pub fn stable_input_hash(input: &str) -> String {
    let mut hasher = StableInputHasher::new();
    hasher.update(input.as_bytes());
    hasher.finalize()
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

    /// Add raw input bytes to the hash.
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

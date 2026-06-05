pub(super) fn hash_to_bit(value: &str, bits: usize) -> usize {
    stable_hash64(value) as usize % bits
}

pub(super) fn stable_hash_hex(value: &str) -> String {
    format!("{:016x}", stable_hash64(value))
}

fn stable_hash64(value: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

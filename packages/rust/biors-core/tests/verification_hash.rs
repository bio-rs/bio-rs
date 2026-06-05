#[test]
fn computes_stable_fixture_hashes() {
    assert_eq!(
        biors_core::verification::stable_input_hash(">seq1\nACDE\n"),
        "fnv1a64:08a331cb13c7bd72"
    );
}

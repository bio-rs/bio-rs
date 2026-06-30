mod common;
mod package_support;

#[test]
fn package_validate_rejects_vocab_with_string_tokens() {
    let value = package_support::validate_package_with_vocab(
        "invalid-vocab-string-tokens",
        r#"{
  "name": "protein-20",
  "unknown_token_id": 20,
  "tokens": ["A", "C"]
}"#,
        None,
    );

    assert_eq!(value["error"]["code"], "package.invalid_vocab_config");
    package_support::assert_invalid_vocab_config_issue(&value, "invalid vocabulary JSON");
}

#[test]
fn package_validate_rejects_vocab_manifest_contract_mismatch() {
    let vocab = package_support::valid_vocab_json();
    let value = package_support::validate_package_with_vocab(
        "invalid-vocab-contract-mismatch",
        &vocab,
        Some(("protein-20-alt", "protein-20-alt.v0")),
    );

    assert_eq!(value["error"]["code"], "package.invalid_vocab_config");
    package_support::assert_invalid_vocab_config_issue(&value, "vocab.name must match");
    package_support::assert_invalid_vocab_config_issue(&value, "vocab.contract_version must match");
}

#[test]
fn package_validate_rejects_vocab_with_wrong_token_order() {
    let value = package_support::validate_package_with_vocab(
        "invalid-vocab-token-order",
        &package_support::valid_vocab_json().replace(r#""token_id": 1"#, r#""token_id": 2"#),
        None,
    );

    assert_eq!(value["error"]["code"], "package.invalid_vocab_config");
    package_support::assert_invalid_vocab_config_issue(&value, "token order and IDs");
}

#[test]
fn package_validate_accepts_dna_iupac_vocab() {
    let output = package_support::package_validate_with_vocab(
        "valid-dna-vocab",
        &package_support::valid_dna_vocab_json(),
        Some(("dna-iupac", "dna-iupac.v0")),
    );

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn package_validate_accepts_rna_iupac_vocab() {
    let output = package_support::package_validate_with_vocab(
        "valid-rna-vocab",
        &valid_rna_vocab_json(),
        Some(("rna-iupac", "rna-iupac.v0")),
    );

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn package_validate_rejects_dna_iupac_vocab_with_wrong_token_order() {
    let value = package_support::validate_package_with_vocab(
        "invalid-dna-vocab-token-order",
        &package_support::valid_dna_vocab_json().replace(r#""token_id": 1"#, r#""token_id": 2"#),
        Some(("dna-iupac", "dna-iupac.v0")),
    );

    assert_eq!(value["error"]["code"], "package.invalid_vocab_config");
    package_support::assert_invalid_vocab_config_issue(&value, "dna-iupac token order and IDs");
}

#[test]
fn package_validate_rejects_dna_iupac_vocab_with_wrong_unknown_id() {
    let value = package_support::validate_package_with_vocab(
        "invalid-dna-vocab-unknown-id",
        &package_support::valid_dna_vocab_json()
            .replace(r#""unknown_token_id": 4"#, r#""unknown_token_id": 20"#),
        Some(("dna-iupac", "dna-iupac.v0")),
    );

    assert_eq!(value["error"]["code"], "package.invalid_vocab_config");
    package_support::assert_invalid_vocab_config_issue(
        &value,
        "unknown_token_id must be 4 for dna-iupac",
    );
}

fn valid_rna_vocab_json() -> String {
    r#"{
  "name": "rna-iupac",
  "tokens": [
    {
      "residue": "A",
      "token_id": 0
    },
    {
      "residue": "C",
      "token_id": 1
    },
    {
      "residue": "G",
      "token_id": 2
    },
    {
      "residue": "U",
      "token_id": 3
    }
  ],
  "unknown_token_id": 4,
  "unknown_token_policy": "warn_or_error_with_unknown_token"
}
"#
    .to_string()
}

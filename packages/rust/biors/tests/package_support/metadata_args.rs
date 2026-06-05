pub fn conversion_metadata_args() -> [&'static str; 16] {
    [
        "--license",
        "CC0-1.0",
        "--citation",
        "bio-rs converted fixture",
        "--model-card",
        "docs/model-card.md",
        "--model-card-summary",
        "Converted package fixture for CLI tests.",
        "--intended-use",
        "CLI conversion test",
        "--limitation",
        "Not for inference",
        "--license-file",
        "docs/LICENSE.txt",
        "--citation-file",
        "docs/CITATION.cff",
    ]
}

pub fn skeleton_metadata_args() -> [&'static str; 10] {
    [
        "--license",
        "CC0-1.0",
        "--citation",
        "bio-rs converted fixture",
        "--model-card-summary",
        "Converted package fixture for CLI tests.",
        "--intended-use",
        "CLI conversion test",
        "--limitation",
        "Not for inference",
    ]
}

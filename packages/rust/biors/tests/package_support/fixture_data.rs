pub const V0_MANIFEST: &str = r#"{
  "schema_version": "biors.package.v0",
  "name": "protein-seed",
  "model": {
    "format": "onnx",
    "path": "models/protein-seed.onnx",
    "checksum": "sha256:2c1da72b15fab35bd6f1bb62f5037b936e26e6413a220fa9afe5a64bce0df68d"
  },
  "tokenizer": {
    "name": "protein-20",
    "path": "tokenizers/protein-20.json",
    "contract_version": "protein-20.v0"
  },
  "vocab": {
    "name": "protein-20",
    "path": "vocabs/protein-20.json",
    "contract_version": "protein-20.v0"
  },
  "preprocessing": [],
  "postprocessing": [],
  "runtime": {
    "backend": "onnx-webgpu",
    "target": "browser-wasm-webgpu"
  },
  "fixtures": [
    {
      "name": "tiny-protein",
      "input": "fixtures/tiny.fasta",
      "expected_output": "fixtures/tiny.output.json"
    }
  ]
}"#;

pub fn valid_dna_vocab_json() -> String {
    r#"{
  "name": "dna-iupac",
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
      "residue": "T",
      "token_id": 3
    }
  ],
  "unknown_token_id": 4,
  "unknown_token_policy": "warn_or_error_with_unknown_token"
}
"#
    .to_string()
}

# biors-mcp-server

Model Context Protocol (MCP) server for [bio-rs](https://github.com/bio-rs/bio-rs) biological sequence processing tools.

## Usage

Run the server over stdio for local agent integration:

```bash
cargo run -p biors-mcp-server
```

Configure your MCP client (Claude Desktop, Cursor, etc.) to launch:

```json
{
  "mcpServers": {
    "bio-rs": {
      "command": "cargo",
      "args": ["run", "-p", "biors-mcp-server"]
    }
  }
}
```

## Tools

- `tokenize` — Tokenize protein, DNA, or RNA FASTA text into stable token IDs
- `validate` — Validate biological sequences (protein, DNA, RNA, or auto-detect)
- `workflow` — Validate → tokenize → model-input workflow with `kind`
  (`auto`, `protein`, `dna`, or `rna`), explicit tokenizer `profile`,
  `max_length`, `pad_token_id`, and `padding` (`fixed_length` or
  `no_padding`) parameters. Explicit kind/profile mismatches are rejected.
- `package_validate` — Validate a package manifest and its filesystem
  artifacts. Pass either `manifest_path` or `manifest_json` plus `base_dir`.
- `package_validate_fields` — Validate only the package manifest JSON fields,
  without filesystem artifact, checksum, or layout checks.
- `doctor` — Report platform readiness diagnostics

## License

MIT OR Apache-2.0

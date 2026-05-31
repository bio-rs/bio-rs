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

- `tokenize` — Tokenize protein FASTA text into stable token IDs
- `validate` — Validate biological sequences (protein, DNA, RNA, or auto-detect)
- `workflow` — Protein-only validate → tokenize → model-input workflow with
  `kind` (`auto` or `protein`), `max_length`, `pad_token_id`, and `padding`
  (`fixed_length` or `no_padding`) parameters
- `package_validate` — Validate a package manifest JSON string
- `doctor` — Report platform readiness diagnostics

## License

MIT OR Apache-2.0

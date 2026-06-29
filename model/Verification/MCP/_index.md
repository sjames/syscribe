---
type: Package
name: MCP
---

Integration test cases for the `syscribe mcp` subcommand — the stdio MCP server, its read
tools, spec/prompt resources, and guarded-write tools. Each case is realised as a black-box
Rust integration test that spawns `syscribe mcp` against a fixture model and drives it over
the MCP stdio protocol.

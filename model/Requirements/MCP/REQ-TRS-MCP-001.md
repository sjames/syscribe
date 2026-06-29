---
type: Requirement
id: REQ-TRS-MCP-001
name: "mcp subcommand starts an MCP server over stdio bound to the model root"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
---

`syscribe -m <root> mcp` shall start a Model Context Protocol server that communicates over
stdio (stdin/stdout), bound to the resolved model root, and shall complete the MCP
`initialize` handshake with a client, advertising its server name, version, and the `tools`,
`resources`, and `prompts` capabilities it supports.

## Transport and lifecycle

- The server reads JSON-RPC messages from stdin and writes responses to stdout; diagnostic
  logging goes to stderr only, so it never corrupts the protocol stream.
- The subcommand resolves the model root the same way every other model-bound command does
  (`--model`/`-m` flag, `$SYSCRIBE_MODEL`, walk-up to `.syscribe.toml`, default `model`).
- The process is long-lived: it serves requests until the client closes stdin, then exits
  with code `0`.

## Protocol version

The server shall negotiate a protocol version compatible with current MCP clients
(Claude Code / Claude Desktop), accepting the version the client offers in `initialize`.

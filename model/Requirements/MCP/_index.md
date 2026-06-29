---
type: Package
name: MCP
---

Requirements for the `syscribe mcp` subcommand: a Model Context Protocol (MCP) server over
stdio that lets LLM clients query and (under guard) mutate the model through a small,
token-efficient tool surface, plus spec/prompt resources.

All requirements derive from `REQ-TRS-MCP-000` and are governed by `ADR-SYS-MCP-001`
(`Decisions::MCPServerADR`).

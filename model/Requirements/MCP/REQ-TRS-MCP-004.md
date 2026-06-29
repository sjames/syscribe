---
type: Requirement
id: REQ-TRS-MCP-004
name: "Format spec exposed as MCP resources and authoring prompts as MCP prompts"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
---

The MCP server shall expose the embedded Syscribe format specification as MCP resources and
the model-authoring prompts as MCP prompts, so an LLM client can pull authoring guidance on
demand rather than guessing the format.

## Resources and prompts

- The format-spec sections embedded in the binary shall be listed and readable as MCP
  resources under stable URIs (e.g. `syscribe://spec/<section>`).
- The create-model authoring prompts embedded in the binary (the `--agent-instructions`
  payloads) shall be exposed as named MCP prompts.
- The server shall declare the `resources` and `prompts` capabilities in its `initialize`
  response so clients enumerate them.

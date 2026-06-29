---
type: Requirement
id: REQ-TRS-MCP-027
name: "Project configuration is exposed as an MCP resource"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - retrieval
---

The server shall expose the project configuration (`.syscribe.toml` — id prefixes, profiles,
links, repos) as an MCP resource, so an LLM can discover project-specific id rules and active
profiles rather than guessing them.

## Behaviour

- A resource at `syscribe://config` shall be listed and readable, returning the project's
  `.syscribe.toml` content.
- When no configuration file is present, the resource shall return an empty/default
  configuration rather than an error.

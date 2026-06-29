---
type: Requirement
id: REQ-TRS-MCP-010
name: "Tool surface is curated, excluding the report/render and feature-model families"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
---

The MCP server shall expose only the curated tool surface — the read tools of REQ-TRS-MCP-003,
the `reload` tool of REQ-TRS-MCP-002, and the write tools of REQ-TRS-MCP-005..007 — and shall
**not** expose the report/render family or the feature-model/projection commands as MCP tools.

## Cut list

- The following CLI command families shall **not** appear in `tools/list`: the report/render
  family (`export`, `plantuml`, `render`, `n2`, `matrix`, `fmea`, `safety-case`, `sbom`,
  `reqif`, `audit`, `metrics`, `zones`) and the feature-model / projection / configuration
  commands (`feature-check`, `configure`, `diff`, and per-configuration projection).
- Token efficiency is the governing constraint: every advertised tool schema is paid for on
  every client call, and the excluded commands either emit large human-formatted documents the
  client can reconstruct from the structured primitives, or are advanced flows deferred to a
  later phase (per `ADR-SYS-MCP-001`).
- The full CLI remains available as the escape hatch for the excluded reports; a guarded
  read-only report passthrough may be added as a separate later requirement.

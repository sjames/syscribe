---
type: Requirement
id: REQ-TRS-MCP-010
name: "Tool surface is curated, excluding the report/render family from dedicated tools"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
---

The MCP server shall expose a curated tool surface and shall **not** expose the report/render
family as dedicated MCP tools. (The feature-model / projection commands, originally excluded,
are now exposed as dedicated tools per REQ-TRS-MCP-028..032; this requirement governs only the
report/render family.)

## Cut list

- The following report/render CLI commands shall **not** appear in `tools/list` as dedicated
  tools: `export`, `plantuml`, `render`, `n2`, `matrix`, `fmea`, `safety-case`, `sbom`,
  `reqif`, `audit`, `metrics`, `zones` (and the rest of the analysis/report family).
- Token efficiency is the governing constraint: every advertised tool schema is paid for on
  every client call, and these commands emit large human-formatted documents the client can
  reconstruct from the structured primitives.
- These commands remain reachable two ways: the full CLI directly, and the guarded read-only
  `run_report` passthrough (REQ-TRS-MCP-033), which runs an allowlisted report command and
  returns its output without adding a dedicated tool per report.

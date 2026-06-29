---
type: Requirement
id: REQ-TRS-MCP-036
name: "coverage_matrix returns the Requirement x Configuration coverage grid"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPEvidenceToolsADR
tags:
  - mcp
---

The MCP server shall expose a `coverage_matrix` read tool returning the Requirement ×
Configuration coverage grid produced by `matrix --json`.

## Behaviour

- The result shall include per-cell state from the unified classifier of REQ-TRS-MCP-044
  (`passing` / `failing` / `covered` / `planned` / `gap` / `na`) — `planned` for a cell covered
  only by a draft TestCase, and the passing/failing/covered split reflecting an ingested results
  sidecar when present — and the `coverage` rollup object (per-Configuration and overall
  covered/applicable counts and percentage; `planned`/`failing`/`gap` count as applicable-but-not-covered).
- Inputs shall support the CLI filters `{ config?, status?, tag?, gaps_only?, linked_only?, plan? }`
  with the same meanings, plus `limit`/`offset`.
- When the model has no feature model, the tool shall fall back to the flat Requirement/TestCase
  view, matching CLI behaviour.

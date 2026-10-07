---
type: PlanningItem
id: PI-TREX-001
name: "Single-document traceability JSON export with configuration projection and sort order"
status: done
itemType: feature
achieves: [REQ-TRS-TREX-001, REQ-TRS-TREX-002, REQ-TRS-TREX-003, REQ-TRS-TREX-004]
evidence:
  - path: repo:crates/syscribe/src/trace_export.rs
  - path: repo:crates/syscribe/tests/trace_export.rs
  - path: repo:crates/syscribe/tests/mcp_read.rs
  - path: repo:prompts/help/trace-export.md
tags:
  - traceability
  - export
---

Delivered 2026-10-08: `trace-export` CLI + `trace_export` MCP tool over one shared computation
(`crates/syscribe/src/trace_export.rs`), the document schema per `REQ-TRS-TREX-001` with the
normative field order, `--config` projection through the existing projection engine, `--sort
directory|asc|desc` on every list, `--out`, the help page, `docs/cli` section and MCP entry, CLI
tests on a temp model, an MCP case on the fixture model and the qualification mirror
`REQ`/`TC-TRS-TREX-001..004`. ADR: `Decisions::TraceExportADR`.

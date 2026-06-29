---
type: Requirement
id: REQ-TRS-MCP-021
name: "apply_changes applies an ordered batch of writes atomically"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - write
---

The MCP server shall expose an `apply_changes` tool that applies an ordered list of write
operations (`create`, `update`, `move`, `delete`) as a single atomic transaction, so an LLM can
make a coherent multi-element change and validate it as a unit.

## Transactional semantics

- The tool shall compute the combined effect of all operations and return a single validation
  delta for the resulting model state.
- The referential-integrity gate (REQ-TRS-MCP-008) and confinement (REQ-TRS-MCP-009) shall be
  evaluated against the **final** state, so an operation that depends on an earlier operation in
  the same batch (e.g. a TestCase that `verifies` a requirement created earlier in the list)
  does not spuriously fail.
- `dry_run` defaults to true. In commit mode the tool shall apply **all** operations or
  **none** — any failure rolls back every change already applied in the batch — and rebuild the
  store only on full success.

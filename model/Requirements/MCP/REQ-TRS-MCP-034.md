---
type: Requirement
id: REQ-TRS-MCP-034
name: "Evidence and diagram tools share a common structured, CLI-parity contract"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPEvidenceToolsADR
tags:
  - mcp
---

The new MCP evidence and diagram/documentation-integrity tools shall observe a common contract.

## Common contract

- Each tool shall return structured JSON with a documented shape — never only human-formatted text.
- A tool that wraps a CLI command shall match that command's `--json` output (CLI parity).
- Read tools shall be side-effect-free and carry `readOnlyHint`; a tool that writes to disk shall
  follow the guarded-write protocol of REQ-TRS-MCP-008 (dry-run default, new-error commit gate with
  the `SYSCRIBE_MCP_ALLOW_NEW_ERRORS` override, store rebuild after commit).
- Findings shall use the canonical syscribe codes (`W010`, `W015`, `W029`, `W099`–`W102`,
  `W400`–`W415`).
- Element references shall be accepted as stable id, qualified name, or display name.
- List/grid-returning tools shall support `limit`/`offset` (or an equivalent bounded response).

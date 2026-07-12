---
type: Requirement
id: REQ-TRS-MCP-045
name: "MCP surfaces suspect-link detection: a read tool and a guarded accept tool"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - suspect-links
---

The MCP server shall expose suspect-link detection (`ADR-SYS-SUSLINK-001`;
`REQ-TRS-SUS-LINKS-*`) so an LLM agent can find and clear stale trace links.

## `suspect_list` (read tool)

- A read-only tool (`read_only_hint = true`) that reports the trace links whose stored
  baseline no longer matches the target's current projection (**suspect**), and, when
  requested, the links that have **no** baseline yet (**unbaselined**).
- Each reported link shall carry the source (id and qualified name), the target reference,
  and the link kind, mirroring `suspect list` (REQ-TRS-SUS-LINKS-006). Output ordering
  shall be deterministic.

## `suspect_accept` (guarded write tool)

- A write tool that records the current state of a link as reviewed by writing the target's
  current projection hash into the source's `traceBaselines` map (REQ-TRS-SUS-LINKS-005).
- It shall obey the common write-guard protocol (REQ-TRS-MCP-008): `dry_run` defaults to
  **true** (no disk write; the would-be change is returned as a diff and a validation
  delta), and a commit rebuilds the store. Because clearing a suspect link **resolves** a
  W090 warning, that resolution shall appear in the delta's `resolvedWarnings`.
- Accepting a `<target>` not referenced by any trace link on `<source>` shall be refused
  with `written: false` and touch no file.
- On a read-only server (`--read-only`), `suspect_accept` shall refuse.

The tools operate over the same projection/hash and detection logic as the CLI, so the CLI
and MCP surfaces can never disagree about whether a link is suspect.

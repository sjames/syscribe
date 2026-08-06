---
type: Requirement
id: REQ-TRS-DE-005
name: "A structural edit that introduces a new referential-integrity error is rejected and surfaced on the diagram"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-DE-000]
breakdownAdr: Decisions::DiagramEditorADR
tags:
  - diagram
---

When a diagram-driven structural edit's guarded-write commit (`REQ-TRS-DE-001`/`002`) would
introduce a new referential-integrity error (a cross-reference that fails to resolve, mirroring
the MCP commit gate), the edit shall be refused: disk is left unchanged, and the diagram client
shall revert the optimistic local change and surface the returned `validationDelta` to the user
inline, rather than the diagram and the on-disk model ever being left inconsistent with each
other, or the file being silently corrupted.

Warnings that are not referential-integrity errors do not block the commit but shall still be
included in the returned `validationDelta` for the user's awareness, consistent with how MCP's
guarded write already treats warnings versus errors.

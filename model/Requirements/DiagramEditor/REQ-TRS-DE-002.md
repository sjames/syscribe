---
type: Requirement
id: REQ-TRS-DE-002
name: "syscribe-server exposes create/delete-element and add/remove-connection endpoints routed through the shared engine"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-DE-000]
breakdownAdr: Decisions::DiagramEditorADR
tags:
  - diagram
---

`syscribe-server` shall expose:

- `POST /api/elements` — create an element (`qname`, `type`, `fields`).
- `DELETE /api/elements/{*qname}` — delete an element, refusing (unless an explicit override is
  given) when other elements still reference it, mirroring the MCP `delete_element` referrer
  check.
- An endpoint to add and one to remove a single entry in an element's `connections:` sequence.

Every one of these shall run through the `REQ-TRS-DE-001` guarded-write engine (dry-run,
candidate validation, commit gated on new referential-integrity errors) and shall return the same
`{validationDelta, diff}` shape MCP's write tools already return, so the calling UI can render
exactly what changed and what (if anything) newly broke.

---
type: Requirement
id: REQ-TRS-DE-003
name: "A diagram's shapes/edges/layout update atomically with edits made from that diagram"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-DE-000]
breakdownAdr: Decisions::DiagramEditorADR
tags:
  - diagram
---

When a structural edit (`REQ-TRS-DE-002`'s endpoints) is made with diagram context — the
originating diagram's qualified name plus a shape or edge id and, for a new shape, a position —
the same guarded-write commit shall also patch that diagram element's `shapes:`, `edges:`, and
`layout:` frontmatter, so that:

- creating an element from within a diagram adds its shape (and layout position) to that diagram,
- deleting an element removes its shape (and any edges referencing it) from every diagram that
  showed it,
- connecting two ports from within a diagram adds the corresponding edge,

in the same commit as the underlying model mutation — never as a separate, independently-fallible
follow-up step that could leave the diagram's view out of sync with the model it depicts.

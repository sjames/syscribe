---
type: Requirement
id: REQ-TRS-DE-004
name: "The web UI renders an editable diagram supporting create-node, delete-node, and connect-edge"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-DE-000]
breakdownAdr: Decisions::DiagramEditorADR
tags:
  - diagram
  - sprotty
---

For `type: Diagram` elements whose `diagram_kind` is a SysML block-style diagram (not
`Mermaid`-kind, which continues to render exactly as it does today), the web UI shall render an
editable `sprotty`-based diagram in the diagram panel (`GET /ui/diagram/{qname}`, whose graph is
loaded from `GET /api/diagrams/model/{qname}`), in place of the former read/reposition-only SVG
canvas for those diagrams, supporting:

- creating a new node (backed by a real model element, via `REQ-TRS-DE-002`/`003`'s endpoints),
- deleting a node,
- connecting two ports by dragging between them,
- repositioning a node (reusing the existing `PATCH /api/diagrams/layout/{qname}` path
  unchanged).

This editable diagram panel is the web UI's only diagram-editing surface. The former
Cytoscape.js graph-explorer view (`GET /canvas`, with `GET /api/graph`) was a separate, purely
analytical view outside this requirement's scope; it was retired in v0.33.0 and has no successor
that this requirement governs.

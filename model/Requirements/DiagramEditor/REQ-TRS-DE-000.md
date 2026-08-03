---
type: Requirement
id: REQ-TRS-DE-000
name: "Users can perform structural model edits directly from the diagram view"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - diagram
  - editor
---

Syscribe shall let a user create, delete, and reconnect model elements directly from the
rendered diagram — not only reposition existing shapes — with every such edit written back to
the underlying Markdown/YAML files and validated the same way any other write path is, so the
diagram becomes a genuine authoring surface rather than a read-only view of the model.

## Rationale

Today's diagram surfaces are view/reposition only: the server-rendered SVG diagrams
(`syscribe-model::renderer`) and the Cytoscape graph explorer let a user look at the model and
drag shapes to a persisted layout (`PATCH /api/diagrams/layout/<qname>`), but every structural
change — adding a part, wiring a connection between two ports, retyping or removing an element —
still requires hand-editing YAML frontmatter, often across more than one file, with no feedback
until the next `syscribe validate` run.

`syscribe-model` already computes the containment tree, the cross-reference/connection graph,
and typed, file+field-located validation findings for the CLI, the web UI, the LSP, and the MCP
server's guarded-write tools (`create_element`/`update_element`/`move_element`,
`ADR-SYS-MCP-001`). A diagram editor should reuse that same structure and the same
propose-then-validate discipline, rather than becoming a second, diagram-only way to mutate the
model that can drift from what the file-based tooling enforces.

## Scope

- In scope: creating a new element from the diagram (backed by a new file in the correct
  namespace directory), deleting an element, and connecting/reconnecting ports (backed by
  `connections:` frontmatter on the owning element).
- Existing capabilities are preserved, not replaced: viewing a diagram and dragging a shape to a
  new layout position continue to work exactly as they do today.
- Every diagram-driven edit is validated by `syscribe-model`'s existing validator before (or
  immediately after) being committed to disk, and a validation failure is surfaced back on the
  diagram rather than silently corrupting the file.
- Which client diagramming framework and edit protocol implement this (sprotty, GLSP, or a
  lighter custom protocol over the existing REST API) is a system-level/architectural decision,
  not part of this requirement.

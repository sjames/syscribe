---
type: Requirement
id: REQ-TRS-LSP-008
name: "Completion offers type-filtered id/qname candidates for cross-reference fields"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPCompletionRenameADR
tags:
  - lsp
  - completion
---

The server shall implement `textDocument/completion`. When the cursor is inside a value of a
known cross-reference field (`supertype`, `subsets`, `redefines`, `typedBy`, `derivedFrom`,
`verifies`, `satisfies`, `breakdownAdr`, `implementedBy`), the server shall offer completion
items limited to ids/qnames of the element type(s) that field expects to resolve to (e.g.
`verifies:` offers only `Requirement` ids; `breakdownAdr:` offers only `ADR` ids).

## Behavior

- Candidates are built from the already-in-memory store; no disk re-read per completion
  request.
- Each candidate's label is the element's stable id if it has one, else its qualified name; the
  candidate's `detail` shows the element's `name`.
- Completion is offered with no trigger characters (the client's default identifier-typing
  completion invokes it) and no `completionItem/resolve` step — every candidate is fully
  populated in the initial response.
- A cursor position outside a known cross-reference field's value offers no field-aware
  candidates (see `REQ-TRS-LSP-009` for the separate enum-field case).

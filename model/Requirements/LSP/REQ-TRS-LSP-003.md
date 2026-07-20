---
type: Requirement
id: REQ-TRS-LSP-003
name: "Go-to-definition resolves qname/id cross-references to their defining file"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPServerADR
tags:
  - lsp
  - navigation
---

The server shall implement `textDocument/definition`. When the cursor is positioned over a
qualified name or stable id used as a cross-reference value (`supertype`, `subsets`,
`redefines`, `typedBy`, `derivedFrom`, `verifies`, `satisfies`, `breakdownAdr`, `traceBaselines`
keys, `implementedBy`, allocation `from`/`to`, and equivalent fields on any element type), the
server shall resolve it through the same resolver `syscribe-model` uses for validation and
return the `Location` of the referenced element's defining file.

## Behavior

- Resolution uses the existing qname/id resolver; both qualified-name and stable-id (`REQ-*`,
  `TC-*`, `ADR-*`, `FEAT-*`, `BL-*`, and any configured `[ids.prefixes]`) forms resolve.
- A reference that does not resolve (dangling — already reported as a validator error/warning
  per `REQ-TRS-LSP-002`) returns no location rather than an error response.
- Cursor positions outside a cross-reference value (e.g. in the Markdown body, or on a YAML key)
  return no location.

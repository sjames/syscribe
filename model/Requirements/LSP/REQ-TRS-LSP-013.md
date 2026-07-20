---
type: Requirement
id: REQ-TRS-LSP-013
name: "codeLens shows display-only reference counts above an element"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPCodeActionsADR
tags:
  - lsp
  - codelens
---

The server shall implement `textDocument/codeLens`. For each element in an open document that
has incoming references (`verifiedBy`, `derivedChildren`, or other reverse-index entries the
validator already computes) or a `W090` suspect-link finding, the server shall return a
`CodeLens` positioned at the element's frontmatter block reporting the relevant counts (e.g.
`2 verifiedBy · 1 derivedChildren`, `1 suspect link`).

## Behavior

- Each `CodeLens` carries a `Command` whose `title` is the display text (the counts) and
  whose `command` identifier is an empty string — a no-op sentinel, since `Command.command`
  is a required field in the LSP type but this phase attaches no click behavior. Drilling
  into the underlying references uses `textDocument/references` (`REQ-TRS-LSP-004`), already
  available.
- Lenses are computed eagerly from the in-memory store when `codeLens` is requested; no
  `codeLens/resolve` step is needed.
- An element with no incoming references and no suspect-link finding gets no lens.

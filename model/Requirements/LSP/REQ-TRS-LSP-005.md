---
type: Requirement
id: REQ-TRS-LSP-005
name: "Hover shows a resolved summary for the qname/id under the cursor"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPServerADR
tags:
  - lsp
  - navigation
---

The server shall implement `textDocument/hover`. When the cursor is positioned over a
resolvable qualified name or stable id (as defined in `REQ-TRS-LSP-003`), the server shall
return a Markdown-formatted summary of the referenced element: its `type`, `id` (if
id-identified), qualified name, `name`, and `status` (if the type carries one).

## Behavior

- The summary is built from the already-parsed element, not by re-reading the file from disk.
- A dangling (unresolvable) reference returns no hover content rather than an error response.
- Hovering over an element's own frontmatter (in its own defining file) returns the same
  summary for that element.

---
type: Requirement
id: REQ-TRS-LSP-009
name: "Completion offers enum-value candidates for type and status fields"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPCompletionRenameADR
tags:
  - lsp
  - completion
---

When the cursor is inside the value of a `type:` field, the server shall offer completion items
for every `ElementType` variant. When the cursor is inside the value of a `status:` field, the
server shall offer completion items for the valid statuses of the enclosing element's type.

## Behavior

- Both candidate sets are sourced from the same domain table that already backs `mcp`'s
  `describe_type`/`template` tools — not a second, independently hand-maintained copy.
- If the enclosing element's type has no defined status domain, no `status:` candidates are
  offered (not an error — an empty completion list).

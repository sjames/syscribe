---
type: Requirement
id: REQ-TRS-LSP-012
name: "rename refuses a candidate that would introduce a new unresolved reference"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPCompletionRenameADR
tags:
  - lsp
  - rename
---

Before returning a `WorkspaceEdit` (`REQ-TRS-LSP-011`), the server shall validate the rename
candidate in memory: apply the rename transform to a clone of the current element list, and
re-run the model validator against the clone.

## Behavior

- If validating the candidate produces a finding that does not appear when validating the
  current (pre-rename) state — i.e. the rename would newly break a cross-reference somewhere —
  `textDocument/rename` returns an LSP error naming the new finding's code and file instead of
  a `WorkspaceEdit`.
- This validation is entirely in-memory: no temporary directory or disk copy is created, since
  nothing is committed to disk by this request.
- A rename that only changes findings the model already had (e.g. an existing unrelated
  warning) is not refused on that basis — only *newly introduced* findings gate the rename.

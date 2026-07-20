---
type: Requirement
id: REQ-TRS-LSP-011
name: "rename computes a WorkspaceEdit across every referencing file; the server never writes to disk"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPCompletionRenameADR
tags:
  - lsp
  - rename
---

The server shall implement `textDocument/rename`. Given a position that `prepareRename`
(`REQ-TRS-LSP-010`) accepts and a new id, it shall return a `WorkspaceEdit` containing:

- a `TextEdit` in the target element's own file, changing its `id:` value to the new id;
- a `TextEdit` in every other file whose frontmatter contains the old id (the same
  needle-matching approach `textDocument/references`, `REQ-TRS-LSP-004`, uses to find
  referencers), changing that occurrence to the new id.

## Behavior

- The server does not write any file itself; the returned edit is applied by the client (to
  open buffers and on-disk files alike, per the `WorkspaceEdit` contract) and saved by the
  client.
- The new id must match the target type's stable-id pattern (e.g. `^REQ(-[A-Z0-9]{2,12})+-[0-9]{3,8}$`
  for a `Requirement`); an id that doesn't match its type's pattern is refused with an LSP
  error before any edit is computed.
- A new id that collides with an existing element's id is refused with an LSP error.

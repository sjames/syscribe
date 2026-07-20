---
type: Requirement
id: REQ-TRS-LSP-007
name: "Model state reloads fully on save and on external file changes"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPServerADR
tags:
  - lsp
---

The server shall keep its in-memory model state consistent with the model files on disk by
performing a full reload (walk, graph rebuild, resolver rebuild) — not incremental
reparsing — whenever the model may have changed.

## Behavior

- `textDocument/didSave` on a file under the model root triggers a full reload.
- `workspace/didChangeWatchedFiles` (registered for the model's `.md` files, covering edits
  made outside the editor — e.g. `git checkout`, another process writing the model) triggers a
  full reload.
- Diagnostics for all currently open documents are recomputed and republished
  (`REQ-TRS-LSP-002`) after every reload.
- A reload that fails (e.g. a transient filesystem error) leaves the previous in-memory state
  in place rather than clearing it.

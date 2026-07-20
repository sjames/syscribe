---
type: Requirement
id: REQ-TRS-LSP-002
name: "Validation findings are published as LSP diagnostics"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPServerADR
tags:
  - lsp
  - diagnostics
---

The server shall run the model validator against the current in-memory model state and publish
its findings as LSP `textDocument/publishDiagnostics` notifications, scoped per open document.

## Behavior

- Diagnostics are (re-)published after `textDocument/didOpen` and after any full model reload
  (`REQ-TRS-LSP-007` — triggered by `textDocument/didSave` or `workspace/didChangeWatchedFiles`).
- Diagnostics are **not** recomputed on `textDocument/didChange` alone: v1 validates the
  on-disk model state, not unsaved buffer content, per `ADR-SYS-LSP-001`'s full-reload,
  no-incremental-parsing decision. An editor sees updated diagnostics once it saves.
- Each diagnostic's `code` is the validator's `E*`/`W*` identifier, its `severity` maps
  `E*` → `Error` and `W*` → `Warning`, and its `message` is the validator's existing
  human-readable text.
- The validator's `Finding` type carries only a file path, not a field/line location, so
  `range` is a best-effort placement: the extent of the element's YAML frontmatter block
  (between its `---` delimiters). Precise per-field ranges are future work, contingent on the
  parser tracking source spans — not a v1 scope item.
- Findings located in a file other than the one that changed (e.g. a dangling cross-reference
  now detected in a file that references the edited element) are published against that other
  file's diagnostics set, not folded into the edited document's.
- No new diagnostic taxonomy is introduced: the LSP layer surfaces exactly the validator's
  existing findings, unfiltered.

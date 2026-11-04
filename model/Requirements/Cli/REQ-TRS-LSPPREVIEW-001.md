---
type: Requirement
id: REQ-TRS-LSPPREVIEW-001
name: "The language server previews a bind or scaffold before it is made, and commits only a clean one"
status: approved
reqDomain: software
reqClass: system
tags:
  - lsp
  - ple
---

The language server shall let a front end see the effect of a mutating action before making it (GH #426):

- the read-only request `syscribe/preview` with `{action: "bind" | "scaffold", …}` plans the action, previews it on a candidate copy through the guarded-write engine and returns the report (diff, findings added and resolved), `commitable`, a `WorkspaceEdit` that makes the change (line-minimal; a created file is a create operation plus its content) and an `undo` edit; it writes nothing;
- the command `syscribe.apply` (`workspace/executeCommand`) commits the same plan only when the preview adds no error, reloads the model, and returns the report and an undo edit that restores the previous lines or deletes a created file; a rejected commit returns an error with the findings and writes nothing.

Applying the offered edit produces exactly the file the CLI writes. Custom requests stay read-only; the single server-side write is the executeCommand route (ADR-SYS-LSP-004 addendum).

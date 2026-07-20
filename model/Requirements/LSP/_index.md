---
type: Package
name: LSP
---

Requirements for the `syscribe lsp` subcommand: a Language Server Protocol server over stdio
that lets LSP-capable editors (VSCode and others) navigate and validate the model — inline
diagnostics, go-to-definition, find-references, hover, and symbol search over the cross-reference
and containment structure `syscribe-model` already computes.

All requirements derive from `REQ-TRS-LSP-000` and are governed by `ADR-SYS-LSP-001`
(`Decisions::LSPServerADR`). Scope here covers the v1 (pure-LSP, no custom protocol extensions)
capability set only; completion/rename (v2) and codeLens/codeAction (v3) are tracked as
follow-on requirements once v1 ships.

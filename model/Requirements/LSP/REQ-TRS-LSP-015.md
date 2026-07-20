---
type: Requirement
id: REQ-TRS-LSP-015
name: "codeAction offers an accept-as-reviewed quick-fix for W090, executed server-side"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPCodeActionsADR
tags:
  - lsp
  - codeaction
---

When the requested range covers a `W090` diagnostic (a stale suspect trace link), the server
shall offer a `CodeAction` titled "Accept as reviewed" whose effect is a `command` referencing
the server-registered `syscribe.suspectAccept` command (advertised via
`executeCommandProvider`), not a `WorkspaceEdit`.

## Behavior

- The server advertises `executeCommandProvider` with `commands: ["syscribe.suspectAccept"]`.
- Handling `workspace/executeCommand` for `syscribe.suspectAccept` calls the same
  `crate::suspect::plan_accept` + `crate::suspect::write_baseline` functions `mcp`'s guarded
  `suspect_accept` tool already uses, writing the new `traceBaselines:` hash to the source
  file's frontmatter directly (no client-applied edit).
- No dry-run step is exposed: opening the codeAction menu and selecting the action is itself
  the confirmation (unlike `mcp`'s `dry_run`-by-default writes, which guard against unreviewed
  LLM-initiated calls).
- After the command runs, the server reloads the store and republishes diagnostics for all open
  documents (`REQ-TRS-LSP-007`'s reload path), so the `W090` finding clears without waiting for
  a filesystem-watch round trip.

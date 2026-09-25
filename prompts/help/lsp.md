# lsp — run a Language Server Protocol server over stdio for editors

`syscribe -m <root> lsp` starts a Language Server (LSP) that speaks
`Content-Length`-framed JSON-RPC 2.0 over **stdio**. It lets an LSP-capable editor
(VSCode and others) navigate and validate the Syscribe model bound at `-m`.

## SYNOPSIS
    syscribe -m <root> lsp

## USAGE

The server runs until it receives `shutdown` followed by `exit` and the client
closes stdin. It is intended to be spawned by an editor's LSP client, not
invoked interactively. It speaks only standard LSP methods — no custom
(non-LSP) protocol extensions (ADR-SYS-LSP-001) — so any LSP-capable client
works, not just a purpose-built extension.

## Capabilities

Advertised in `initialize`:


- **Diagnostics** (`textDocument/publishDiagnostics`) — the model validator's
  findings (`E***`/`W***`), republished on `textDocument/didOpen` and after a
  full model reload.
- **Go to definition** (`textDocument/definition`) — resolves the qualified
  name or stable id under the cursor to its defining file.
- **Find references** (`textDocument/references`) — every element whose
  frontmatter cross-references the element under the cursor.
- **Hover** (`textDocument/hover`) — a resolved summary (type, id, qname,
  status) for the qualified name or stable id under the cursor.
- **Workspace symbol search** (`workspace/symbol`) — find elements by name,
  id, or qualified name across the whole model.
- **Completion** (`textDocument/completion`, ADR-SYS-LSP-002) — field-aware
  id/qname candidates inside cross-reference fields (`supertype`/`subsets`/
  `redefines` → same type, `typedBy` → the matching `*Def`, `satisfies`/
  `verifies`/`derivedFrom` → `Requirement`, `breakdownAdr` → `ADR`), plus enum
  candidates for `type:` and `status:`.
- **Rename** (`textDocument/prepareRename` + `textDocument/rename`) — rename a
  stable id (id-identified elements only) and every reference to it. The server
  never writes: it returns a `WorkspaceEdit` for the client to apply, after
  validating the result in memory (a malformed or colliding id is refused).
  Renaming a qualified name is a file move — use `move` or the MCP
  `move_element` tool.
- **Code lens** (`textDocument/codeLens`, ADR-SYS-LSP-003) — a display-only lens
  on the frontmatter: `N verifiedBy · N derivedChildren · N suspect links`.
- **Code actions** (`textDocument/codeAction`, quick fixes) — for `E310` (a
  derived Requirement with no `breakdownAdr:`), one "Set breakdownAdr: …" edit
  per accepted ADR; for `W090` (suspect link), an "Accept as reviewed" command.
- **Execute command** (`workspace/executeCommand`) — `syscribe.suspectAccept
  <source> <target>` re-baselines one suspect link (the only LSP path that
  writes to disk), then reloads and republishes diagnostics.

The model state is disk-backed and reloads in full on `textDocument/didSave`
and `workspace/didChangeWatchedFiles` — `textDocument/didChange` alone never
triggers a reload or diagnostics (the server validates saved content, not
unsaved buffers).

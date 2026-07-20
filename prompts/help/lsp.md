# lsp — run a Language Server Protocol server over stdio for editors

`syscribe lsp -m <model>` starts a Language Server (LSP) that speaks
`Content-Length`-framed JSON-RPC 2.0 over **stdio**. It lets an LSP-capable editor
(VSCode and others) navigate and validate the Syscribe model bound at `-m`.

## Usage

    syscribe -m <model> lsp

The server runs until it receives `shutdown` followed by `exit` and the client
closes stdin. It is intended to be spawned by an editor's LSP client, not
invoked interactively. It speaks only standard LSP methods — no custom
(non-LSP) protocol extensions (ADR-SYS-LSP-001) — so any LSP-capable client
works, not just a purpose-built extension.

## v1 capabilities

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

The model state is disk-backed and reloads in full on `textDocument/didSave`
and `workspace/didChangeWatchedFiles` — `textDocument/didChange` alone never
triggers a reload or diagnostics (v1 validates saved content, not unsaved
buffers).

Completion, rename, codeLens, and codeAction are not implemented yet (planned
for a later phase per `ADR-SYS-LSP-001`) and are not advertised in
`initialize`.

---
type: ADR
id: ADR-SYS-LSP-001
name: "Expose the model to editors via a Language Server (syscribe lsp), pure LSP, phased capability rollout"
status: accepted
tags:
  - lsp
  - editor
  - tooling
---

## Context

VSCode (and other LSP-capable editors) are commonly used to view and author Syscribe model
files directly. Today that editing experience is generic Markdown/YAML: no awareness that a
`derivedFrom:`/`verifies:`/`supertype:` value is a cross-reference into another file, no inline
validation feedback, and no safe way to rename a stable id without hand-grepping every
referencing file.

The model's own structure already makes most of this cheap to provide: stable ids and
qualified names are the cross-reference currency, `syscribe-model` already computes the trace
graph, reverse indices (`verifiedBy`, `derivedChildren`, ...), and typed, file+field-located
validation findings (E*/W* codes). `crates/syscribe/src/mcp/store.rs` (`McpStore`, per
`ADR-SYS-MCP-001`) already shows the exact blueprint — `walk_model` → `build_graph` →
`Resolver` → `ValidateConfig` — reused by both `syscribe-server` and `syscribe mcp`. A
Language Server is a third, editor-facing consumer of that same store.

Axes evaluated:

- **Protocol** — implement only standard LSP capabilities vs. also define custom JSON-RPC
  methods (à la rust-analyzer) for model-specific actions the LSP spec doesn't cover (e.g.
  "open this element's diagram" against the running `syscribe-server`).
- **Placement** — a new `syscribe-lsp` crate vs. a `syscribe lsp` subcommand, mirroring the
  `mcp` subcommand precedent.
- **SDK** — `tower-lsp` (async, Tokio-native) vs. hand-rolled JSON-RPC over `lsp-types`.
- **Reload strategy** — full model reload on save/external change (matching
  `McpStore::reload`) vs. incremental re-parsing.
- **Capability scope for v1** — the full editing surface (diagnostics, definition, references,
  hover, symbol search, completion, rename, codeLens, codeAction) at once vs. a phased subset.

## Decision

A new **`syscribe lsp`** subcommand runs a **Language Server over stdio**, built on
**`tower-lsp`**, exposing **standard LSP capabilities only** — no custom protocol extensions.

- **Protocol: pure LSP, no custom methods.** Every feature surfaces through a standard LSP
  request/notification. This keeps the server usable from any LSP-capable client (VSCode,
  Neovim, Zed, Helix, ...), not just a bespoke extension, and keeps the VSCode extension a thin
  generic `vscode-languageclient` bootstrap that spawns the binary over stdio — no custom
  view/panel code.
- **Placement: a subcommand in the `syscribe` CLI crate**, following the `mcp` subcommand
  precedent (`ADR-SYS-MCP-001`) — reuses the store/query/validation internals directly rather
  than wiring a fourth crate.
- **SDK: `tower-lsp`**, async and Tokio-native like the rest of the workspace (`axum`,
  `notify`); the subcommand builds a local tokio runtime the same way `mcp` does, `main` stays
  synchronous otherwise.
- **Reload: full model reload**, not incremental parsing. `textDocument/didSave` and
  `workspace/didChangeWatchedFiles` (for edits made outside the editor, e.g. `git checkout`)
  both trigger a full `reload()` of the store, mirroring `McpStore::reload`. Incremental
  parsing is deferred until reload latency is actually a problem.
- **Capability scope, phased:**
  - **v1** — `publishDiagnostics`, `textDocument/definition`, `textDocument/references`,
    `textDocument/hover`, `workspace/symbol`. These require no new domain logic — the
    validator and resolver already produce exactly this data; the server is an LSP-shaped
    adapter over data `syscribe-model` already computes.
  - **v2** — `textDocument/completion` (id/qname completion for cross-reference fields, enum
    completion for `status`/`testLevel`/`reqDomain`) and `textDocument/rename` (safe stable-id
    rename across all referencing files, reusing the `mcp` write path's frontmatter
    round-trip).
  - **v3** — `codeLens` (inline `verifiedBy`/`derivedChildren`/suspect-link counts) and
    `codeAction` (insert missing `breakdownAdr`, run `suspect accept`, stub a `Baseline`).

## Rationale

**Pure LSP over custom protocol extensions.** A custom method (e.g. a "show diagram" request)
would only ever work from a purpose-built VSCode extension, and any such action is expressible
instead as a standard `codeAction`/`command` whose handler opens a `file://`/`http://` URI —
sufficient for the diagram-launch case without protocol drift. Portability across editors and
a minimal, generic client outweigh the convenience of a bespoke request type.

**v1 scope: diagnostics/definition/references/hover/symbol first.** These map directly onto
existing `syscribe-model` outputs (validator findings, resolver, reverse indices, `ftsearch`)
with essentially no new logic — the highest value-per-line-of-new-code slice. Completion and
rename need new logic (schema-aware suggestion, safe multi-file rewrite) and codeLens/codeAction
need UI-facing design; both are deferred rather than gating the first usable release.

**Subcommand over new crate.** Same reasoning as `ADR-SYS-MCP-001`: ships in the binary users
already have, reuses the CLI's existing query/validation/resolver code directly, avoids a
third (now fourth) crate's build/dependency wiring.

**`tower-lsp` over hand-rolled JSON-RPC.** Async-native and fits the workspace's existing
Tokio usage (`axum`, `notify`) instead of introducing a synchronous LSP loop alongside async
code elsewhere in the crate.

**Full reload over incremental parsing.** `McpStore::reload` already establishes that a full
walk+rebuild is cheap enough for these model sizes; matching that keeps the LSP's store logic
identical to the MCP store's rather than inventing a second, more complex incremental scheme
with no evidence it's needed yet.

## Consequences

- A new dependency (`tower-lsp`, and its `lsp-types`) enters the `syscribe` crate; the `lsp`
  subcommand builds a local tokio runtime the same way `mcp` does.
- This is the **third** independent implementation of the walk→graph→resolver→config store
  blueprint (`syscribe-server`, `syscribe mcp`, now `syscribe lsp`). Extracting a shared
  `ModelStore` into `syscribe-model` becomes worth doing once this third consumer exists, but
  is not a prerequisite for v1 — the `lsp` subcommand may start by duplicating `McpStore`'s
  shape and the extraction can follow as a separate refactor.
- Diagnostics reuse the validator's existing E*/W* codes and file+field locations as-is; no new
  diagnostic taxonomy is introduced by the LSP layer.
- v2's `rename` reuses the `mcp` write path's frontmatter round-trip (preserves unknown keys
  and body text on edit) rather than a new writer.
- No HTTP/SSE or other non-stdio transport is introduced; the server is stdio-only, matching
  how every current LSP client (including VSCode's `vscode-languageclient`) launches local
  language servers.
- Subsequent requirements build on this decision in phases: v1 capabilities
  (`REQ-TRS-LSP-002`..`REQ-TRS-LSP-006`), with v2 (completion, rename) and v3 (codeLens,
  codeAction) requirements to follow once v1 ships.

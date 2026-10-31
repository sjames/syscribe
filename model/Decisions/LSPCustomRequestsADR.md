---
type: ADR
id: ADR-SYS-LSP-004
name: "LSP custom requests — namespaced, read-only syscribe/* requests for model views that standard LSP cannot express"
status: accepted
tags:
  - lsp
  - editor
  - tooling
  - variability
---

## Context

`ADR-SYS-LSP-001` decided that `syscribe lsp` exposes **standard LSP capabilities only**, so
that any LSP client works and the VS Code extension stays a thin `vscode-languageclient`
bootstrap. That fits authoring features (diagnostics, hover, completion, rename, quick fixes).

It does not fit the product-line front end. A feature-model tree, a configurator, residual
variability by stage, a requirement-by-configuration coverage matrix, a variant diff and a
change-impact view are **structured model views**, not documents, symbols or edits. Standard LSP
has no request for them, and encoding them as hovers, code lenses or `workspace/executeCommand`
results works only for the smallest cases. Today the front end shells out to the CLI and
parses text, which is brittle, slow per call, and cannot be notified when the model changes.

Axes evaluated:

- **Transport for structured views** — keep standard-LSP-only and use the CLI's `--json`;
  allow custom LSP requests; or have the extension speak MCP to `syscribe mcp`.
- **Write path** — whether custom requests may mutate the model.
- **Compatibility** — how an extension copes with a binary that predates a request.
- **Duplication** — whether the CLI and the LSP return different shapes for the same view.

## Decision

`syscribe lsp` may define **custom, namespaced, read-only requests** for model views that no
standard LSP method can express. Standard LSP remains the way to deliver everything it can
express; a custom request is added only when it cannot.

- **Namespace.** Every custom method is named `syscribe/<view>` (for example
  `syscribe/featureModel`, `syscribe/configure`, `syscribe/residual`, `syscribe/matrix`).
  Custom notifications use the same prefix (for example `syscribe/modelChanged`).
- **Read-only.** A custom request never writes a file. Mutations remain a `WorkspaceEdit`
  returned to the client, or `workspace/executeCommand` through the guarded-write path
  (`ADR-SYS-LSP-002/003`, `syscribe_model::mutate`), unchanged.
- **Versioned payloads.** Each result is JSON with a `schemaVersion`, and its schema is
  published and checked in CI. A breaking change bumps the version.
- **Advertised.** The server lists the custom methods it supports, with their schema
  versions, in the `initialize` result under `capabilities.experimental.syscribe` (and via
  `syscribe capabilities --json`). A client sends a custom request only if it is advertised.
- **One payload, two transports.** The CLI `--json` form of a view and the LSP request return
  the same typed payload, produced by one function in `syscribe-model` or the `syscribe`
  crate. Neither is derived from the other's text output.
- **Standard clients keep working.** The server stays fully usable from a client that ignores
  every custom method: diagnostics, hover, definition, completion, rename, code lens and code
  action are unchanged and need no custom method.

`ADR-SYS-LSP-001` is **amended**, not superseded: its transport, placement, SDK, reload and
phasing decisions stand. Only its "no custom methods" clause is relaxed, as above.

## Rationale

**Custom requests over CLI-only `--json`.** The CLI starts a process and reloads the model on
every call. The LSP server already holds a loaded, validated store, so a request is cheap, can
be cancelled, and can be paired with a change notification so panels refresh without polling.

**Custom requests over MCP.** MCP is built for agents and tool calls; the extension already
holds one language-server connection and a reload lifecycle (`didSave`, watched files). A
second connection would duplicate the store and its reload timing.

**Read-only and namespaced.** A prefix keeps custom methods from colliding with standard or
other servers' methods, and read-only keeps the safety property that made the pure-LSP
decision attractive: the server cannot change a model behind the user's back.

**Advertise, don't assume.** Extensions are updated independently of the binary. A handshake
lets the front end hide a panel, not fail, when the installed binary is older.

**One payload.** Two shapes for the same view would drift. Producing both from one typed value
keeps the CLI usable in scripts and CI with exactly what the editor sees.

## Consequences

- The "only standard LSP methods" statements in `docs/cli/index.md`, `prompts/help/lsp.md` and
  `editors/vscode/README.md` stay true until the first custom request ships; the first one to
  ship updates them in the same change.
- Each custom request needs its own requirement and a `crates/syscribe/tests/lsp_*.rs` test in
  the existing harness (`tests/common/mod.rs`), plus a schema check.
- Clients other than the VS Code extension may implement the custom views, but they are not
  required to; portability of the standard capabilities is unchanged.
- The server gains a small request-routing layer on top of `tower-lsp` for custom methods.
- Large views (the matrix, the feature tree of a big product line) need a size budget and,
  where a result can be big, pagination; each request states its own budget.
- Follow-on work is tracked in the PLE front-end issues (#411, #413 JSON contract, #414
  capabilities handshake, #415 change notifications).

---
type: ADR
id: ADR-SYS-DE-001
name: "Diagram-driven structural editing: sprotty client over a shared, syscribe-model-hosted guarded-write engine (no GLSP)"
status: accepted
tags:
  - diagram
  - editor
  - sprotty
  - mcp
---

## Context

`REQ-TRS-DE-000` asks for diagrams that are a genuine authoring surface — create, delete, and
reconnect elements from the rendered diagram — not just a view. Today's diagram surfaces
(`syscribe-model::renderer`'s server-rendered SVG canvas, and the separate read-only Cytoscape.js
graph explorer) support viewing and drag-to-reposition only. Structural changes still mean
hand-editing YAML frontmatter, often across more than one file, with no feedback until the next
`syscribe validate` run.

Three axes were evaluated:

- **Client diagramming framework** — hand-rolled SVG editing, extending the existing Cytoscape.js
  view with edit plugins (`cytoscape-edgehandles` etc.), or adopting `sprotty` (Eclipse's
  diagramming framework). And if `sprotty`: paired with the full **GLSP** (Graphical Language
  Server Platform) protocol, or used standalone.
- **Where the write logic lives** — `syscribe-server`'s `routes/write.rs` today does its own
  unguarded "read file, patch YAML mapping, rewrite file" (three independent copies of this
  pattern exist across the codebase, none sharing logic, none validating before commit), while
  `crates/syscribe`'s MCP server already has a complete guarded-write engine
  (`create_element`/`update_element`/`move_element`/`delete_element`, each running through
  `guarded_write`: candidate-copy the tree, apply, re-validate, diff baseline vs. candidate, and
  gate the real commit on not introducing a new referential-integrity error). The question is
  whether the diagram editor gets its own third implementation, or the existing one is shared.
- **`connections:`/`features:` representation** — introduce a typed schema for port
  bindings/attributes, or keep them as the untyped `serde_yaml::Value` sequences they are today
  and add narrow typed helpers only where a diagram edit needs to mutate them.

## Decision

1. **Extract the MCP guarded-write engine into `syscribe-model`.** `write_confined`,
   `element_ref_strings`/`ref_errors`, `guarded_write` (candidate-copy/apply/re-validate/diff/
   commit-gate), and the stable-id allocator move out of `crates/syscribe/src/mcp/*` into
   `syscribe-model`, operating on a model root path and element list rather than MCP's JSON tool
   types. A single `patch_frontmatter(content, |mapping| ...) -> String` helper in
   `syscribe-model::frontmatter` replaces the three independent "split frontmatter, mutate
   mapping, reassemble" copies. `crates/syscribe/src/mv.rs`'s qname validation and textual
   reference-rewrite logic moves alongside it. MCP's tool handlers and `syscribe-server`'s
   `write.rs` both become thin callers of this one engine.
2. **`connections:`/`features:` stay untyped YAML.** No new schema. `syscribe-model` gains a
   small typed helper — parse/add/remove one entry in a `connections:` sequence — that follows
   the dotted-endpoint conventions `graph.rs::resolve_endpoint` already implements, so a
   diagram-added connection resolves identically to a hand-written one.
3. **A diagram's `shapes:`/`edges:`/`layout:` frontmatter update transactionally with the model
   edit.** Creating a Part from inside a diagram creates the Part file and adds its shape/layout
   entry to that diagram in the same guarded-write call — one atomic commit, not a model edit
   followed by a separate, potentially-failing view-sync step.
4. **Client: `sprotty`, standalone — not the GLSP protocol.** Sprotty renders and provides the
   edit gestures (move, create-edge, delete) as local Actions; custom action handlers intercept
   these, call the new REST endpoints, and revert the optimistic local change if the server
   rejects the edit, surfacing the returned validation delta. No diagram-server protocol, no new
   long-lived process.
5. **New endpoints on the existing `syscribe-server`**, not a new server/crate: `POST /api/elements`,
   `DELETE /api/elements/{*qname}`, and connection add/remove, all routed through the engine from
   (1), all returning the MCP-style `{validationDelta, diff}` shape.

## Rationale

**Extract over duplicate.** `syscribe-server`'s current write path has no validation gate at
all — a diagram edit could silently corrupt the model. MCP's engine already solves exactly this
problem correctly (dry-run by default, commit gated on referential integrity, not on every
validator warning). Writing a third version for the diagram editor would triple the surface that
can drift; extracting it once into `syscribe-model` (which both the CLI/MCP and the server already
depend on) means every write path — CLI, MCP, LSP code actions, and the diagram editor — shares
one tested implementation.

**Sprotty over extending Cytoscape, and standalone over GLSP.** Cytoscape's edit plugins exist
but the current Cytoscape view is a separate, purely analytical graph explorer (dagre-laid-out,
lane-snapped) with a different visual model than the SVG block diagrams `renderer.rs` produces;
retrofitting structural editing onto it would mean editing the wrong diagram. Sprotty is built
for exactly this — editable, structured diagrams with a real command/undo stack — but GLSP (the
protocol normally paired with it) is Java/Node-oriented with no Rust SDK; implementing it here
would mean hand-rolling a second bespoke protocol server (after the LSP one) for no benefit this
project needs. Sprotty predates GLSP and was designed to work standalone against a plain backend,
which is all a REST-based diagram editor requires.

**Untyped `connections:`/`features:` over a new schema.** These are hand-authored today and a
schema migration would be a breaking change to every existing model file for a feature that only
needs to *add and remove entries*, not restructure them. A narrow typed helper gets the diagram
editor what it needs (a real function instead of ad hoc YAML surgery) without touching the format.

**Transactional diagram sync over separate steps.** A diagram editor whose "create" leaves the
new element invisible until a second, independently-fallible sync step succeeds is not actually a
diagram editor — it would routinely leave the view and the model out of sync. Bundling both
mutations into one guarded-write commit means the feature is either fully applied or not applied.

## Consequences

- `syscribe-model` gains a write/mutation responsibility it did not have before (previously
  parse + validate + render only); this is a deliberate scope expansion, tracked by
  `REQ-TRS-DE-001`.
- `crates/syscribe/src/mcp/*` and `mv.rs` shrink to thin wrappers over the extracted engine; the
  existing `mcp_*.rs` integration test suite is the regression gate proving behavior didn't shift.
- The project's first JS/TS build step is introduced (a small `esbuild` bundle for the sprotty
  client) — dev-time only; the served artifact remains a plain static file through the existing
  `rust_embed` pipeline, same as the vendored `cytoscape.min.js` today.
- Out of scope for this decision, tracked as `REQ-TRS-DE-006` follow-on work: live multi-client
  sync over the currently-unconsumed `/ws` broadcast channel, and hosting the same editor inside
  the VSCode webview (the extension is a pure LSP client today, with no webview infrastructure).

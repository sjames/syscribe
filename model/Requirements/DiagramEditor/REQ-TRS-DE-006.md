---
type: Requirement
id: REQ-TRS-DE-006
name: "Follow-on: live multi-client diagram sync and VSCode-webview hosting"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-DE-000]
breakdownAdr: Decisions::DiagramEditorADR
tags:
  - diagram
  - future
---

Two capabilities shall be tracked as explicitly deferred follow-on scope past the first cut of
diagram-driven structural editing, not built as part of `REQ-TRS-DE-001`..`005`:

- **Live multi-client sync.** `syscribe-server`'s `/ws` broadcast channel exists today but has no
  client consumer at all; a future iteration should have the diagram client subscribe to it and
  refresh on external changes, with an edit-echo suppression mechanism (there is none to adapt
  today, since nothing currently listens) so a client doesn't visibly "flicker" on its own commit.
- **VSCode-webview hosting.** The same editor should eventually be embeddable inside a webview
  panel in `editors/vscode/`, which today is a pure LSP client with no webview infrastructure.

Reparenting, retyping, and structural editing of element types beyond `Part`/`PartDef`/
`Connection` are likewise out of scope for the first cut and are not tracked as a separate
requirement here pending a concrete need.

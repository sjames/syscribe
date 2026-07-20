---
type: Requirement
id: REQ-TRS-LSP-000
name: "Editors can navigate and validate the model via a Language Server"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - lsp
  - editor
---

Syscribe shall provide a Language Server Protocol (LSP) interface so that editors (VSCode and
other LSP-capable editors) can offer inline validation, cross-reference navigation, and symbol
search over the model, without the author hand-tracing qualified names and stable ids through
raw Markdown/YAML files.

## Rationale

VSCode is commonly used to view and author Syscribe model files directly, but today that
experience is generic Markdown/YAML editing: no indication that a `derivedFrom:`/`verifies:`/
`supertype:` value is a cross-reference into another file, no inline feedback from the
validator while editing, and no safe way to find every reference to an element before changing
it.

`syscribe-model` already computes the cross-reference graph, reverse indices, and typed,
file+field-located validation findings for the `syscribe-server` web UI and the `syscribe mcp`
LLM interface. An LSP interface makes that same structure available directly inside the editor
the author is already using.

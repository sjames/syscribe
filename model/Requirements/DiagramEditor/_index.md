---
type: Package
name: DiagramEditor
---

Requirements for diagram-driven structural editing of the model: creating, deleting, and
reconnecting elements from the rendered diagram (not just viewing it or repositioning existing
shapes), with edits written back to the underlying Markdown/YAML files and validated the same
way any other write path (`syscribe validate`, the LSP, the MCP guarded-write tools) is.

All requirements derive from `REQ-TRS-DE-000` and are governed by `ADR-SYS-DE-001`
(`Decisions::DiagramEditorADR`). The scope covers a shared guarded-write engine in
`syscribe-model`, create/delete/connection endpoints on `syscribe-server`, transactional
diagram-view sync, a standalone `sprotty`-based editable diagram client (not GLSP), and a
reject-and-surface discipline for edits that would break referential integrity. Live
multi-client sync and VSCode-webview hosting are follow-on scope, not built in the first cut.

The member requirements are listed by `syscribe show Requirements::DiagramEditor` (generated from
this directory — not maintained here).

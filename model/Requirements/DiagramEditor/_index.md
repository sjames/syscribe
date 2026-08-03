---
type: Package
name: DiagramEditor
---

Requirements for diagram-driven structural editing of the model: creating, deleting, and
reconnecting elements from the rendered diagram (not just viewing it or repositioning existing
shapes), with edits written back to the underlying Markdown/YAML files and validated the same
way any other write path (`syscribe validate`, the LSP, the MCP guarded-write tools) is.

All requirements derive from `REQ-TRS-DE-000` and are governed by `ADR-SYS-DE-001`
(`Decisions::DiagramEditorADR`): a shared guarded-write engine extracted from the MCP server into
`syscribe-model` (`REQ-TRS-DE-001`), new create/delete/connection endpoints on `syscribe-server`
routed through it (`REQ-TRS-DE-002`), transactional diagram-view sync (`REQ-TRS-DE-003`), a
`sprotty`-based editable diagram client — standalone, not GLSP (`REQ-TRS-DE-004`), and a
reject-and-surface discipline for edits that would break referential integrity
(`REQ-TRS-DE-005`). Live multi-client sync and VSCode-webview hosting are tracked as follow-on
scope in `REQ-TRS-DE-006`, not built in this first cut.

---
id: REQ-TRS-BT-005
type: Requirement
name: "binding-times lists the effective vocabulary and is surfaced to authoring agents"
status: draft
reqDomain: software
verificationMethod: test
---

`binding-times [--json]` shall list the effective vocabulary (name, order, description, binder); the MCP server shall expose a read-only equivalent; and `--agent-instructions` shall append a "Project binding times" section so an authoring agent never invents a stage.

**Acceptance criteria:**
- `binding-times --json` output is stable and ordered earliest to latest.
- The agent-instructions text lists the project's declared stages.

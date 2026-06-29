---
type: Requirement
id: REQ-TRS-MCP-009
name: "Write tools confine all file operations to the resolved model root"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - write
  - security
---

Every MCP write tool (`create_element`, `update_element`, `move_element`) shall confine all
file creation, modification, and deletion to the resolved model root, and shall reject any
qualified name, destination, or path that would resolve to a location outside that root.

## Confinement guards

- A requested qualified name or destination whose path would escape the model root — for
  example via `..` segments, an absolute path, or a symlink that points outside the root —
  shall be rejected with `written: false` and a descriptive error, before any disk change.
- No file shall ever be created, modified, moved, or deleted outside the resolved model root
  as a result of any tool call, in either `dry_run` or commit mode.
- This guard applies in addition to the basic-name / id-scheme identity checks of
  REQ-TRS-MCP-005 and the common write-guard protocol of REQ-TRS-MCP-008; it is enforced even
  when a commit is otherwise permitted.

Because an LLM client supplies the qualified names directly, treating them as untrusted input
is essential: a path-traversal write would let a model-authoring session corrupt files outside
the model entirely, bypassing the validation gate that only inspects in-model findings.

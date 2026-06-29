---
type: Requirement
id: REQ-TRS-MCP-018
name: "Task-oriented MCP prompts encode the project's authoring conventions"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - authoring
---

The MCP server shall expose task-oriented MCP prompts that guide an LLM through the project's
authoring conventions, beyond the general `create-model` prompt (REQ-TRS-MCP-004).

## Prompts

- The server shall offer at least `add-requirement`, `break-down-requirement`,
  `add-testcase-for`, and `traceability-review` prompts.
- Each prompt shall encode the relevant conventions so the produced artifacts are
  convention-correct — for example: a derived requirement carries `derivedFrom` and a
  `breakdownAdr` pointing at an `accepted` ADR with an appropriate `reqClass`; a TestCase
  `verifies` its requirement and uses a stable `TC-*` id and a valid `testLevel`.
- Prompts that take a target element shall accept it as a prompt argument.

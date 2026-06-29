---
type: Requirement
id: REQ-TRS-MCP-016
name: "Tools carry MCP annotations distinguishing read-only from mutating behaviour"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
---

The MCP server shall annotate each tool with MCP tool annotations that declare its behaviour,
so a client can render and guard tools appropriately.

## Annotations

- Every read/query tool (`search`, `get_element`, `list_by_type`, `tree`, `neighbors`,
  `graph_query`, `trace`, `impact`, `validate`, `validate_element`, `reload`, and the authoring
  helpers `describe_type`, `template`, `explain_finding`, `check_ref`, `next_id`, `coverage`)
  shall carry `readOnlyHint: true`.
- The write tools (`create_element`, `update_element`, `move_element`) shall not carry
  `readOnlyHint: true`; they shall be annotated as non-destructive (they create/modify rather
  than delete) so a client knows they mutate the model.
- The annotations shall be present in the `tools/list` output.

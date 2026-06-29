---
type: Requirement
id: REQ-TRS-MCP-014
name: "explain_finding tool explains a validation code and how to resolve it"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - authoring
---

The MCP server shall expose an `explain_finding` tool that, given a validation code (an `E*`,
`W*`, or `I*` code as returned by `validate`/`validate_element`), returns a human-readable
explanation of the rule and guidance on how to resolve it.

## Behaviour

- The explanation shall be sourced from the embedded format specification so it stays
  consistent with the validator.
- This closes the self-correction loop: after `validate` reports a finding, the LLM can look
  up the code and apply a fix rather than guessing.
- An unrecognised code shall return a structured error.

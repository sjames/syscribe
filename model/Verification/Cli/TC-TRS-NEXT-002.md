---
id: TC-TRS-NEXT-002
type: TestCase
testLevel: L3
status: active
name: "Verify the MCP tools next_item and done_check are read-only and return the CLI data."
verifies:
  - REQ-TRS-NEXT-001
  - REQ-TRS-DONE-001
sourceFile: repo:crates/syscribe/tests/mcp_planning.rs
testFunctions:
  - next_item_and_done_check_are_read_tools
---

```gherkin
Feature: MCP next_item and done_check (TC-TRS-NEXT-002)

  Scenario: The tools are read-only and match the CLI
    Given an MCP server over a model with planning items
    When a client calls next_item and done_check
    Then they return the next item and what is missing, and nothing on disk changes
```

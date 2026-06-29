---
type: TestCase
id: TC-TRS-MCP-016
name: "Tools carry read-only vs mutating annotations"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_authoring.rs
verifies:
  - REQ-TRS-MCP-016
tags:
  - mcp
---

```gherkin
Feature: Tool annotations

  Scenario: read tools are annotated read-only and write tools are not
    Given an initialized mcp server
    When tools/list is requested
    Then get_element carries readOnlyHint true
    And validate carries readOnlyHint true
    And create_element does not carry readOnlyHint true
    And update_element does not carry readOnlyHint true
```

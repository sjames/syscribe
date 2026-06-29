---
type: TestCase
id: TC-TRS-MCP-023
name: "Write tools return a unified text diff of changed files"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_write.rs
verifies:
  - REQ-TRS-MCP-022
tags:
  - mcp
  - write
---

```gherkin
Feature: Write diff preview

  Scenario: a dry-run update returns a unified diff
    Given an initialized mcp server over a fixture model
    When update_element changes a field on REQ-FX-001 without committing
    Then the result includes a diff identifying the element's file
    And the diff shows the field's old and new values as removed/added lines
    And the file on disk is unchanged
```

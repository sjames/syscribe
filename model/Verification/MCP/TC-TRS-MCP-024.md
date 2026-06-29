---
type: TestCase
id: TC-TRS-MCP-024
name: "Read-only mode hides the write tools but keeps the read surface"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_live.rs
verifies:
  - REQ-TRS-MCP-023
tags:
  - mcp
  - security
---

```gherkin
Feature: Read-only mode

  Scenario: write tools are absent under --read-only
    Given the server is started with --read-only over a fixture model
    When tools/list is requested
    Then none of create_element, update_element, move_element, delete_element, apply_changes are listed
    And get_element, search, and validate are still listed

  Scenario: the default mode still exposes write tools
    Given the server is started without --read-only
    Then create_element is listed
```

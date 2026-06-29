---
type: TestCase
id: TC-TRS-MCP-007
name: "move_element relocates an element and rewrites inbound references"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_write.rs
verifies:
  - REQ-TRS-MCP-007
tags:
  - mcp
  - write
---

Verifies that `move_element` renames/relocates an element to a new qualified name and updates
references to it across the model.

```gherkin
Feature: Element move with reference rewriting

  Scenario: a referenced element is moved and references follow
    Given a fixture where element B references element A
    When move_element relocates A to a new qname with dry_run=false
    Then A's file exists at the new path and not the old one
    And B's reference to A now names the new qname
    And the tool reports the rewritten files

  Scenario: an invalid destination qname is rejected
    When move_element is called with a malformed destination qname
    Then the call reports written=false and the source is unchanged
```

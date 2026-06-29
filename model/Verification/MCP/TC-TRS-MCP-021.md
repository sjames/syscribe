---
type: TestCase
id: TC-TRS-MCP-021
name: "delete_element removes an element and is blocked by inbound references"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_write.rs
verifies:
  - REQ-TRS-MCP-020
tags:
  - mcp
  - write
---

```gherkin
Feature: Guarded element deletion

  Scenario: deleting a referenced element is refused
    Given a fixture where Derived references Base via supertype
    When delete_element is called for Base with dry_run=false and no force
    Then the call reports written=false and lists the blocking inbound reference from Derived
    And Base's file still exists

  Scenario: deleting an unreferenced element succeeds
    Given an element with no inbound references
    When delete_element is called for it with dry_run=false
    Then the call reports written=true and the file is removed

  Scenario: force deletes despite references
    When delete_element is called for Base with force=true and dry_run=false
    Then the call reports written=true and Base's file is removed
```

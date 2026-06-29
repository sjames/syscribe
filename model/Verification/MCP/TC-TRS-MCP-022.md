---
type: TestCase
id: TC-TRS-MCP-022
name: "apply_changes applies an ordered batch atomically"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_write.rs
verifies:
  - REQ-TRS-MCP-021
tags:
  - mcp
  - write
---

```gherkin
Feature: Transactional batch writes

  Scenario: a dependent pair commits as one unit
    Given an initialized mcp server over a fixture model
    When apply_changes commits a create of a new requirement followed by a create of a
      test case that verifies it, in one ordered batch with dry_run=false
    Then the call reports written=true
    And both new elements exist on disk
    And the batch returns a single combined validation delta

  Scenario: a failing batch rolls back fully
    When apply_changes commits a batch whose second operation is invalid, with dry_run=false
    Then the call reports written=false
    And the first operation's file was not left on disk
```

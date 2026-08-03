---
type: TestCase
id: TC-TRS-MCP-048
name: "delete_element and the referential-integrity gate resolve references nested in Allocation features: entries"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/src/mutate/guard.rs
verifies:
  - REQ-TRS-MCP-047
tags:
  - mcp
  - write
  - allocation
---

```gherkin
Feature: Reference scan sees allocations nested in Allocation features: entries

  Scenario: deleting an element referenced only via a nested allocation is blocked
    Given an Allocation element whose features: list has an entry with
      allocatedTo pointing at Target
    When delete_element is called for Target with dry_run=false and no force
    Then the call reports written=false
    And blockedBy lists the Allocation element
    And Target's file still exists

  Scenario: force still deletes despite a nested allocation reference
    When delete_element is called for Target with force=true and dry_run=false
    Then the call reports written=true and Target's file is removed

  Scenario: a nested allocatedFrom/allocatedTo pointing at nothing raises EREF
    Given an Allocation element whose features: list has an entry with
      allocatedTo pointing at a nonexistent qname
    When the referential-integrity gate scans the model
    Then an EREF finding is reported for the `allocatedTo` field on the Allocation element
```

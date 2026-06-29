---
type: TestCase
id: TC-TRS-MCP-008
name: "Write tools honour dry-run, return a validation delta, and gate on new errors"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_write.rs
verifies:
  - REQ-TRS-MCP-008
tags:
  - mcp
  - write
---

Verifies the common write-guard protocol: `dry_run` defaults true and never touches disk, a
validation delta is returned, and a commit that would introduce a new error is refused.

```gherkin
Feature: Guarded write protocol

  Scenario: dry_run leaves disk byte-for-byte unchanged
    Given an initialized mcp server over a fixture model
    And a snapshot hash of the fixture directory
    When update_element is called without dry_run (defaulting to true)
    Then the response describes the would-be change
    And the fixture directory hash is unchanged

  Scenario: the validation delta is reported
    When a write tool is called for a change that would resolve or add findings
    Then the response includes the newly introduced and newly resolved findings

  Scenario: a commit introducing a new error is refused
    Given a change that would introduce a new error-severity finding
    When the write tool is called with dry_run=false and no override
    Then the call reports written=false and the offending new error in the delta
    And disk is unchanged

  Scenario: a clean commit rebuilds the store
    When a valid change is committed with dry_run=false
    Then a subsequent read reflects the change without an explicit reload
```

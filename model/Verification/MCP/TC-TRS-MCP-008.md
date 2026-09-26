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

  Scenario: errors the gate does not cover are still reported, flagged non-gating (GH #187)
    Given a change that introduces E310 (a derived requirement with no breakdownAdr)
    When the write tool is called with dry_run=true and again with dry_run=false
    Then newErrors contains E310 with gating=false and the commit succeeds
    And an EREF or E630-E636 error carries gating=true

  Scenario: resolving a non-gating error is reported
    When a change supplies the missing breakdownAdr
    Then resolvedErrors contains E310

  Scenario: paths leaving the model root resolve as in the real tree (GH #186)
    Given a [plantuml] style_file of "../style.iuml" that exists beside the model root
    When a write tool is called
    Then the delta reports no W415, new or resolved

  Scenario: a finding present before and after cancels out (GH #186)
    Given a style_file that does not exist
    When a write tool is called
    Then the pre-existing W415 appears in neither newWarnings nor resolvedWarnings

  Scenario: the staging directory is removed and an unwritable parent falls back to the temp dir
    When writes are dry-run, committed and refused
    Then no .syscribe-mcp-cand-* directory remains beside the model
    And with an unwritable parent the guard still returns a delta

  Scenario: a clean commit rebuilds the store
    When a valid change is committed with dry_run=false
    Then a subsequent read reflects the change without an explicit reload
```

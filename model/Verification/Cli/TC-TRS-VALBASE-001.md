---
type: TestCase
id: TC-TRS-VALBASE-001
name: "validate gates only on findings new relative to a baseline"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/model_changes_and_new_findings.rs
verifies:
  - REQ-TRS-VALBASE-001
tags:
  - validation
---

```gherkin
Feature: New-findings gate

  Scenario: pre-existing warnings against a new one
    Then --new-since HEAD exits 0 for the accepted warnings and 2 once one more is added

  Scenario: file moved
    Then its findings stay pre-existing

  Scenario: JSON output
    Then every finding carries new true or false, and the field is absent without a baseline

  Scenario: baseline file round trip
    Then --write-baseline then --baseline-file gates identically

  Scenario: conflicting or invalid options
    Then they exit 1
```

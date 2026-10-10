---
type: TestCase
id: TC-TRS-DOCFIX-001
name: "status rule, coverage legend and CAL3 wording are present in command output"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/doc_fixes_output.rs
verifies:
  - REQ-TRS-DOCFIX-001
tags:
  - docs
---

```gherkin
Feature: output states its rules

  Scenario: verification-depth
    Then the report names the counted TestCase status

  Scenario: testplan
    Then the table is followed by a legend saying what Coverage is a percentage of

  Scenario: W039 for CAL3
    Then the message says I2 or higher
```

---
type: TestCase
id: TC-TRS-SCBACK-001
name: "safety-case text collapses repeated subtrees"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/safety_case_backref.rs
verifies:
  - REQ-TRS-SCBACK-001
tags:
  - safety
---

```gherkin
Feature: safety-case back-references

  Scenario: shared requirement
    Given two arguments citing the same requirement that has a verifying test
    Then the requirement line appears twice, the second marked (see above), and the test once

  Scenario: json
    Then the JSON still contains the subtree under both arguments
```

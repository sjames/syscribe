---
type: TestCase
id: TC-TRS-AUDITCLS-001
name: "audit lists coverage per requirement class"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/audit_class_coverage.rs
verifies:
  - REQ-TRS-AUDITCLS-001
tags:
  - cli
---

```gherkin
Feature: audit coverage by class

  Scenario: per-class counts
    Then the text and JSON carry complete/partial/none counts and the percentage per reqClass

  Scenario: policy and errors
    Then a rollup rule changes a stakeholder verdict and an invalid table is reported in the section
```

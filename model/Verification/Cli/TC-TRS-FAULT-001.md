---
type: TestCase
id: TC-TRS-FAULT-001
name: "Injectable failure modes: schema, engine kinds and scenario steps"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/fault_injection.rs
verifies:
  - REQ-TRS-FAULT-001
tags:
  - fault-injection
---

```gherkin
Feature: Fault injection

  Scenario: schema
    Then injection on FMEA rows and fault-tree events is accepted, and malformed items are E936

  Scenario: engine kinds
    Then stuck, offset, silent, drop, delay and corrupt behave as specified and can be cleared except silent

  Scenario: scenario steps
    Then injected and cleared steps change the run, are recorded in the verdict, and an unknown fault id is E937
```

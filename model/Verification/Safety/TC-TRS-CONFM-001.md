---
type: TestCase
id: TC-TRS-CONFM-001
name: "confirms accepts the analysis and plan work products and still rejects other types"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/confirmation_targets.rs
verifies:
  - REQ-TRS-CONFM-001
tags:
  - safety
---

```gherkin
Feature: confirms targets

  Scenario: analysis and plan work products
    Then confirms naming a FaultTree, FMEASheet, TARASheet, ADR, Argument, TestPlan or Allocation raises no E860

  Scenario: other types
    Then confirms naming a PartDef raises E860 listing the accepted types
```

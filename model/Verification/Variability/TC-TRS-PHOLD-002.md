---
type: TestCase
id: TC-TRS-PHOLD-002
name: "binding changes drift only the matching config-scoped baseline"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/placeholders_baseline.rs
verifies:
  - REQ-TRS-PHOLD-002
tags:
  - variability
---

```gherkin
Feature: placeholder bindings and baselines

  Scenario: config-scoped baselines
    Given baselines frozen for configuration X and for configuration Y
    When X's parameter binding changes
    Then verifying the baseline for X fails and for Y passes

  Scenario: base model
    Given a full-model baseline
    When a binding changes
    Then it still verifies

  Scenario: detail
    Then diff --current names the element whose substituted text changed
```

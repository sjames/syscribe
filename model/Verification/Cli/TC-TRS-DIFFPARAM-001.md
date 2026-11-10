---
type: TestCase
id: TC-TRS-DIFFPARAM-001
name: "diff reports differing parameter values with source and unit"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/diff_params.rs
verifies:
  - REQ-TRS-DIFFPARAM-001
tags:
  - cli
---

```gherkin
Feature: parameter differences between two Configurations

  Scenario: same features, different bindings
    Then the feature sections are empty and the parameters section lists only the differing parameter with both values, sources and the unit

  Scenario: --all and a value only one side has
    Then every parameter is listed with differs marked, and a missing value is null, validated against the diff schema

  Scenario: identical Configurations
    Then parameters is an empty list
```

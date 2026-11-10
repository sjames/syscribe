---
type: TestCase
id: TC-TRS-MOECOMB-001
name: "MoE key checks, combined MoEs and Celsius"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/moe_combined_units.rs
verifies:
  - REQ-TRS-MOECOMB-001
tags:
  - cli
  - magicgrid
---

```gherkin
Feature: MoE checks, combined MoEs and Celsius

  Scenario: a misspelt key
    Then MG034 names the unknown key, suggests the right one and reports the missing key and the weight sum

  Scenario: an expression over several measurements
    Then the MoE value is the expression over the recorded values

  Scenario: a MoE built from other MoEs
    Then the inputs roll up, a leaf knock-out propagates with its reason and only the top MoE is weighted

  Scenario: Celsius
    Then 23.4 degC is 296.55 K and a difference of two Celsius values is in kelvin
```

---
id: TC-TRS-BT-021
type: TestCase
testLevel: L3
status: active
name: "Verify the source and stage of each effective value is determined"
verifies:
  - REQ-TRS-BT-021
---

```gherkin
Feature: Value provenance
  Scenario: inherited value
    Given a value bound by a derivedFrom base
    When provenance is computed
    Then it names the base and its stage

  Scenario: default and propagated
    Given a defaulted value and a bindTo-propagated value
    When provenance is computed
    Then they report default and the bindTo source respectively
```

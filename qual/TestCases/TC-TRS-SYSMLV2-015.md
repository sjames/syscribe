---
id: TC-TRS-SYSMLV2-015
type: TestCase
testLevel: L3
status: active
name: "Verify a dotted connect endpoint keeps its full path and raises no W542 (truncation retired by GH #206), resolving an inherited tail without a finding."
verifies:
  - REQ-TRS-SYSMLV2-015
---

```gherkin
Feature: dotted connect endpoints are not truncated
  Scenario: a non-redeclared two-segment endpoint raises no W542 and resolves
    Given a connect clause whose two-segment endpoints are not redeclared on either head
    When the model is validated
    Then no W542 fires and the inherited port tail resolves

  Scenario: a redeclared two-segment endpoint raises no W542
    Given a connect clause whose two-segment endpoints are redeclared on both heads
    When the model is validated
    Then no W542 fires

  Scenario: a bare endpoint raises no W542
    Given a connect clause with bare, undotted endpoints
    When the model is validated
    Then no W542 fires

  Scenario: a three-segment endpoint raises no W542
    Given a connect clause with a three-segment endpoint chain
    When the model is validated
    Then no W542 fires
```

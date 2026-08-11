---
id: TC-TRS-SYSMLV2-015
type: TestCase
testLevel: L3
status: draft
name: "Verify a genuinely two-segment, non-redeclared connect endpoint raises W542 for each truncated end, while a redeclared endpoint, a bare endpoint, and a three-segment endpoint all raise none."
verifies:
  - REQ-TRS-SYSMLV2-015
---

```gherkin
Feature: W542 connect-endpoint truncation warning
  Scenario: a non-redeclared two-segment endpoint raises W542 for each truncated end
    Given a connect clause whose two-segment endpoints are not redeclared on either head
    When the model is validated
    Then W542 fires exactly twice, one per truncated endpoint

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

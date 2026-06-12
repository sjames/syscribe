---
id: TC-TRS-PARAM-003
type: TestCase
testLevel: L3
status: draft
name: "Verify inclusive range syntax min..=max is enforced (E205)."
verifies:
  - REQ-TRS-PARAM-003
---

```gherkin
Feature: inclusive parameter range syntax
  Scenario: an out-of-range binding against a 1..=8 range is E205
    Given a parameter declared range "1..=8" and a configuration binding 99
    When the model is validated
    Then E205 is reported
  Scenario: a binding at the inclusive upper bound is accepted
    Given the same parameter and a configuration binding 8
    Then no E205 is reported
```

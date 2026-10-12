---
id: TC-TRS-PLANT-001
type: TestCase
testLevel: L3
status: draft
name: "Verify in-process PlantDef plants, library plants and plant faults."
verifies:
  - REQ-TRS-PLANT-001
---

```gherkin
Feature: in-process plants
  Scenario: a lag in a closed loop runs and is deterministic
    Given a controller bound to a first-order lag PlantDef and an FMEA with a parameter and a stuck-output fault
    When the tool runs `validate` and `simulate`
    Then no error is reported and every scenario passes, including the analytic step response and both faults
    And two runs give the same JSON output
  Scenario: malformed plants are reported
    Given a derivative of the wrong dimension, an algebraic loop, an unknown library plant and a plant key on a part definition
    When the tool runs `validate`
    Then E935, E938, E939 and E938 are reported
```

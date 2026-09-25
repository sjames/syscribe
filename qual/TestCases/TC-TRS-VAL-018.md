---
id: TC-TRS-VAL-018
type: TestCase
testLevel: L3
status: draft
name: "Verify E924 (ConfirmationMeasure status not planned/in_progress/completed), E925 (targetSL/achievedSL outside 1-4) and E926 (Zone/Conduit status not draft/review/approved/deprecated)."
verifies:
  - REQ-TRS-VAL-018
---

```gherkin
Feature: Documented enumerations and ranges are enforced (TC-TRS-VAL-018)

  Scenario: a ConfirmationMeasure status outside the documented set raises E924
    Given a ConfirmationMeasure with status: approved and another with status: completed
    When the tool validates the model
    Then E924 names the 'approved' measure and its value
    And no E924 names the 'completed' measure

  Scenario: a Security Level outside 1-4 raises E925
    Given a Zone with achievedSL 7, a Conduit with achievedSL 0 and a PartDef with targetSL 5
    When the tool validates the model
    Then E925 is reported for each of the three values
    And a Zone whose levels are 1-4 raises no E925

  Scenario: a Zone or Conduit status outside the documented set raises E926
    Given a Zone with status: active and a Conduit with status: done
    When the tool validates the model
    Then E926 names each value
    And a clean Zone/Conduit model raises none of E924-E926
```

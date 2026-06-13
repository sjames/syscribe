---
id: TC-TRS-SAFE-009
type: TestCase
testLevel: L3
status: draft
name: "Verify W039 fires for silLevel 3 and 4 SafetyGoals missing I3 assessment"
verifies:
  - REQ-TRS-SAFE-009
---

```gherkin
Feature: W039 independence check extended to IEC 61508 SIL 3 and SIL 4

  Scenario: silLevel 4 goal without I3 assessment raises W039
    Given a model with a SafetyGoal carrying silLevel 4 and at least one ConfirmationMeasure present
    And no I3 functional_safety_assessment confirms that goal
    When the tool validates
    Then W039 is raised naming the SafetyGoal

  Scenario: silLevel 3 goal without I3 assessment raises W039
    Given a model with a SafetyGoal carrying silLevel 3 and the same ConfirmationMeasure
    When the tool validates
    Then W039 is raised naming the SafetyGoal

  Scenario: silLevel 2 goal does not raise W039
    Given a model with a SafetyGoal carrying silLevel 2
    When the tool validates
    Then W039 is not raised for this goal

  Scenario: asilLevel D still triggers W039 — regression
    Given a model with a SafetyGoal carrying asilLevel D, same conditions
    When the tool validates
    Then W039 is raised naming the SafetyGoal

  Scenario: no ConfirmationMeasure — opt-in dormant
    Given a model with only a silLevel 4 SafetyGoal and no ConfirmationMeasure
    When the tool validates
    Then W039 is not raised
```

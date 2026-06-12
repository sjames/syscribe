---
id: TC-TRS-SAFE-007
type: TestCase
testLevel: L3
status: draft
name: Verify W038 work-product responsibility, the ConfirmationMeasure type with W039 independent-assessment coverage, the E84x structural errors, and the opt-in rules
verifies:
  - REQ-TRS-SAFE-007
---

Verify that the tool flags a non-draft work product with no `responsibility:` (W038) once
DIA/CIA tracking is adopted, that it flags an ASIL-D item lacking its required I3
functional-safety assessment (W039) once confirmation tracking is adopted, that an invalid
`ConfirmationMeasure` enum yields E84x, that both checks are dormant when their practice is
not adopted, and that `--deny W038`/`--deny W039` gate.

```gherkin
Feature: Confirmation measures, assessment independence, and DIA/CIA responsibility

  Scenario: W038 fires on a non-draft work product with no responsibility
    Given a model where at least one element declares responsibility
    And another non-draft work product declares no responsibility
    When the tool validates the model
    Then a W038 finding is emitted naming the work product
    And the model validates with no errors

  Scenario: assigning a responsibility clears W038
    Given the same model where every non-draft work product declares responsibility
    When the tool validates the model
    Then no W038 finding is emitted
    And the model validates with no errors

  Scenario: W039 fires on an ASIL-D goal lacking its I3 functional-safety assessment
    Given an asilLevel D SafetyGoal
    And at least one ConfirmationMeasure exists but none is an I3 functional_safety_assessment confirming it
    When the tool validates the model
    Then a W039 finding is emitted naming the missing assessment
    And the model validates with no errors

  Scenario: an I3 functional-safety assessment clears W039
    Given the same ASIL-D SafetyGoal
    And a ConfirmationMeasure with measureType functional_safety_assessment, independenceLevel I3, confirms the goal
    When the tool validates the model
    Then no W039 finding is emitted
    And the model validates with no errors

  Scenario: an invalid ConfirmationMeasure enum yields E84x
    Given a ConfirmationMeasure with an invalid measureType and independenceLevel
    When the tool validates the model
    Then E849 and E850 findings are emitted

  Scenario: both checks are dormant without their practice adopted
    Given a model with no responsibility and no ConfirmationMeasure anywhere
    When the tool validates the model
    Then no W038 finding is emitted
    And no W039 finding is emitted

  Scenario: --deny W038 and --deny W039 make validation exit non-zero
    Given the flagged models
    When the tool validates with --deny W038 (resp. --deny W039)
    Then the tool exits with a non-zero status
```
</content>

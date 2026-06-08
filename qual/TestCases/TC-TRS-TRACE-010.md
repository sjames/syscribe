---
id: TC-TRS-TRACE-010
type: TestCase
testLevel: L3
status: draft
title: "Verify the unsatisfied safety-mechanism check W306 (high-integrity + draft/unsatisfied/all-N-A)."
verifies:
  - REQ-TRS-TRACE-010
---

Verify that W306 fires for a high-integrity requirement that is draft, unsatisfied, or active in no configuration, names the sub-condition, is silent below the threshold and when fully integrated, and is gateable.

```gherkin
Feature: unsatisfied safety-mechanism check

  Scenario: high-integrity draft requirement produces W306
    Given a silLevel 4 requirement with status draft
    When the tool validates the model
    Then a W306 finding is emitted naming the draft sub-condition

  Scenario: high-integrity unsatisfied requirement produces W306
    Given a silLevel 4 approved requirement that no element satisfies
    When the tool validates the model
    Then a W306 finding is emitted naming the unsatisfied sub-condition

  Scenario: a fully-integrated high-integrity requirement produces no W306
    Given a silLevel 4 approved requirement satisfied by an element
    When the tool validates the model
    Then no W306 finding is emitted for it

  Scenario: a requirement below the integrity threshold never produces W306
    Given a silLevel 2 draft, unsatisfied requirement
    When the tool validates the model
    Then no W306 finding is emitted for it

  Scenario: W306 is gateable
    Given a model with a W306
    When the tool validates with --deny W306
    Then the tool exits non-zero
```

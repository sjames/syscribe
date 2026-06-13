---
id: TC-TRS-SAFE-011
type: TestCase
testLevel: L3
status: draft
name: "Verify safety-case suppresses implicit fold-in per-goal and globally via --no-implicit"
verifies:
  - REQ-TRS-SAFE-011
---

```gherkin
Feature: safety-case implicit fold-in suppression

  Scenario: goal with explicit Argument does not show implicit requirements fold-in
    Given a model with SafetyGoal SG-A that has a supporting Argument ARG-001 in its support list
    And a Requirement REQ-X with derivedFromSafetyGoal: SG-A and a verifying TestCase TC-X
    When the user runs safety-case
    Then the output for SG-A does not contain the implicit [evidence:Requirement] section

  Scenario: goal without Argument still shows implicit fold-in
    Given a model with SafetyGoal SG-B that has no supporting Argument
    And a Requirement REQ-Y with derivedFromSafetyGoal: SG-B
    When the user runs safety-case
    Then the output for SG-B contains the implicit requirements section

  Scenario: --no-implicit suppresses fold-in for all goals
    Given a model with SafetyGoal SG-A (with Argument) and SG-B (without Argument)
    When the user runs safety-case --no-implicit
    Then neither SG-A nor SG-B output contains the implicit requirements fold-in

  Scenario: JSON omits requirements entries for goals with explicit Arguments
    Given SafetyGoal SG-A with a supporting Argument
    When the user runs safety-case --json
    Then the goal SG-A entry in the JSON does not contain a "requirements" array
```

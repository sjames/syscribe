---
id: TC-TRS-SAFE-011
type: TestCase
testLevel: L3
status: draft
name: "Verify safety-case folds in the implicit chain for every goal, exposes completeness and rejects an unknown goal"
verifies:
  - REQ-TRS-SAFE-011
---

```gherkin
Feature: safety-case implicit fold-in and completeness

  Scenario: goal with explicit Argument still folds in uncited derived requirements
    Given a SafetyGoal SG-A with a supporting Argument that cites REQ-X
    And a Requirement REQ-Y with derivedFromSafetyGoal: SG-A that the Argument does not cite
    When the user runs safety-case
    Then the output for SG-A shows REQ-Y as an implicit requirement
    And REQ-X is shown once, under the Argument

  Scenario: goal without Argument still shows implicit fold-in
    Given a model with SafetyGoal SG-B that has no supporting Argument
    And a Requirement REQ-Z with derivedFromSafetyGoal: SG-B
    When the user runs safety-case
    Then the output for SG-B contains the implicit requirement

  Scenario: --no-implicit suppresses fold-in for all goals
    Given a model with SafetyGoal SG-A (with Argument) and SG-B (without Argument)
    When the user runs safety-case --no-implicit
    Then neither SG-A nor SG-B output contains a direct implicit requirement

  Scenario: JSON lists implicit requirements
    Given SafetyGoal SG-A with a supporting Argument and an uncited derived requirement
    When the user runs safety-case --json
    Then the goal SG-A entry lists the uncited requirement under "requirements" with implicit true

  Scenario: completeness summary
    When the user runs safety-case and safety-case --json
    Then the text ends with a Completeness summary
    And the JSON has a completeness object and a per-goal verdict

  Scenario: unknown goal id
    When the user runs safety-case SG-NOPE-999
    Then the tool exits with a non-zero exit code
```

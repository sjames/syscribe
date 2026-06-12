---
id: TC-TRS-SAFE-008
type: TestCase
testLevel: L3
status: draft
name: Verify the GSN argument layer (Argument ARG-*, AssumptionOfUse AOU-*), the E852–E858/W040 checks, and the safety-case (GSN) view incl. the implicit goal→req→test fold-in
verifies:
  - REQ-TRS-SAFE-008
---

Verify that the tool recognises the GSN argument-layer types `Argument` and
`AssumptionOfUse`, validates them (required id/title/status, id patterns, `argumentType`
enum, ref resolution, orphan warning), and renders the `safety-case` view in both text and
JSON form, including the implicit `SafetyGoal → Requirement → TestCase` fold-in.

```gherkin
Feature: GSN argument layer and safety-case view

  Scenario: a valid GSN model validates with no errors
    Given a SafetyGoal, a strategy Argument supporting it, a sub-Argument and a Requirement
    And an AssumptionOfUse that appliesTo the goal
    When the tool validates the model
    Then the model validates with no errors

  Scenario: safety-case text shows the argument tree and the AoU
    Given the same valid GSN model
    When the tool renders the safety-case view as text
    Then the output shows the SafetyGoal, its [strategy] and [claim] arguments, evidence leaves, and the [AoU]

  Scenario: safety-case --json is valid JSON with arguments and assumptions
    Given the same valid GSN model
    When the tool renders the safety-case view as JSON
    Then the output is valid JSON exposing goals[].arguments and goals[].assumptions

  Scenario: an unresolved Argument.supports yields E855
    Given an Argument whose supports names a non-existent element
    When the tool validates the model
    Then an E855 finding is emitted

  Scenario: an orphan claim Argument yields W040
    Given a claim Argument with no supports and no evidence
    When the tool validates the model
    Then a W040 finding is emitted

  Scenario: the implicit fold-in works without explicit Argument nodes
    Given a model with SafetyGoals, Requirements (derivedFromSafetyGoal) and verifying TestCases
    When the tool renders the safety-case view
    Then each SafetyGoal shows its derived Requirements and their verifying TestCases
```

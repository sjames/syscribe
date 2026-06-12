---
id: TC-TRS-SEC-003
type: TestCase
testLevel: L3
status: draft
name: "Verify ISO/SAE 21434 attack tree types: AttackTree/AttackTreeGate/AttackStep, weakest-link roll-up, E915–E921, W035."
verifies:
  - REQ-TRS-SEC-003
---

Verify that the tool recognises the `AttackTree`/`AttackTreeGate`/`AttackStep`
element types, rolls up attack feasibility with the weakest-link rule (AND=min
along a path, OR=max across alternatives), reconciles the computed value with the
linked `ThreatScenario.attackFeasibility` via a gateable `W035`, and enforces the
structural checks (E915–E921) and the empty-tree warning, mirroring the FTA
family.

```gherkin
Feature: ISO/SAE 21434 attack tree types and weakest-link feasibility roll-up

  Scenario: a well-formed attack tree validates with no errors
    Given an AttackTree AT-DEMO-001 with threatRef to ThreatScenario TS-DEMO-001,
      a root OR gate ATG-DEMO-001 over an AND gate ATG-DEMO-002 and a step ATS-DEMO-003,
      with steps nested in the tree's subdirectory
    When the tool validates the model
    Then validation reports zero errors

  Scenario: the roll-up matches the worked example and is reconciled against the threat
    Given AT-DEMO-001 whose ATG-DEMO-002 (AND) has steps high and low -> min = low,
      and whose root OR has [ATG-DEMO-002 (low), ATS-DEMO-003 (medium)] -> max = medium,
      while TS-DEMO-001 declares attackFeasibility high
    When the tool validates the model
    Then exactly one W035 finding naming computed medium and declared high is emitted

  Scenario: aligning the declared feasibility clears W035
    Given the same tree but TS-DEMO-001 declares attackFeasibility medium
    When the tool validates the model
    Then no W035 finding is emitted and there are no errors

  Scenario: a threatRef to a non-ThreatScenario produces E917
    Given an AttackTree whose threatRef points at an element that is not a ThreatScenario
    When the tool validates the model
    Then an E917 finding is emitted

  Scenario: W035 is gateable with --deny
    Given the worked-example model with a W035
    When the tool validates with --deny W035
    Then the tool exits non-zero

  Scenario: the new types are recognised and dangling-ref clean
    Given the worked-example model
    When the tool validates the model
    Then no orphan or dangling-reference errors are emitted for the attack tree elements
    And the tool's `types` command lists AttackTree, AttackTreeGate and AttackStep
```

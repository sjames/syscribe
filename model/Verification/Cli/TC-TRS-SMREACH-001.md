---
type: TestCase
id: TC-TRS-SMREACH-001
name: "dead and trap state warnings see transitions at other levels of the hierarchy"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/state_reach.rs
verifies:
  - REQ-TRS-SMREACH-001
tags:
  - state-machine
  - validation
---

```gherkin
Feature: W070 and W071 across the state hierarchy

  Scenario: entered only by a transition declared on its parent
    Given a composite whose own transition targets one of its children
    When the model is validated
    Then no W070 is raised for that child

  Scenario: entered only from a nested state
    Given a nested state whose transition targets a state outside its composite
    When the model is validated
    Then no W070 is raised for the target

  Scenario: left by the parent's transition
    Given a leaf whose parent has the only outgoing transition
    When the model is validated
    Then no W071 is raised for the leaf

  Scenario: a path target names the state
    Given a transition whose target is written Live.Parked
    When the model is validated
    Then no W070 is raised for Parked

  Scenario: real dead and trap states are still reported
    Given a state no transition enters and a state no transition leaves
    When the model is validated
    Then W070 and W071 name them
```

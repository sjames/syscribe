---
id: TC-TRS-SM-001
type: TestCase
testLevel: L3
status: draft
name: "Verify the canonical SysMLv2 transition schema: nested (implicit source) and top-level (explicit source) placements yield the same edges; accept string and {payload} forms are equivalent; both validate clean."
verifies:
  - REQ-TRS-SM-001
---

Verify the tool recognises one canonical transition schema regardless of authoring
placement, by validating minimal `StateDef` fixtures.

```gherkin
Feature: Canonical SysMLv2 state-transition schema

  Scenario: nested per-substate transitions validate clean
    Given a StateDef whose substates carry their own transitions: with target/accept/guard
    When the tool validates the model
    Then no state-machine completeness warning (W070–W074) is emitted

  Scenario: top-level transitions with explicit source validate clean
    Given a StateDef with a top-level transitions: list using source/target
    When the tool validates the model
    Then no state-machine completeness warning (W070–W074) is emitted

  Scenario: accept string shorthand equals the payload map form
    Given a transition using accept: Items::Cmd and another using accept: {payload: Items::Cmd}
    When the tool validates the model
    Then both are accepted and neither raises a non-determinism warning on its own

  Scenario: nested and top-level placements produce the same machine
    Given the same state machine authored once nested and once top-level
    When the tool validates both models
    Then both are equally clean of state-machine warnings
```

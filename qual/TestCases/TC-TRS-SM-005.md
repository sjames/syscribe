---
id: TC-TRS-SM-005
type: TestCase
testLevel: L3
status: draft
name: "Verify composite/hierarchical state machines: recursion into inner regions (region-named W073), composite-as-node top-level checks, and W076 for unresolved transition endpoints; clean composite is silent; draft-suppressed; gateable."
verifies:
  - REQ-TRS-SM-006
  - REQ-TRS-SM-007
---

Verify the hierarchy-aware checks for composite state machines and unresolved transition
endpoints.

```gherkin
Feature: Composite state machine validation (recursion + W076)

  Scenario: well-formed composite machine is clean
    Given a composite StateDef whose top level and inner region are both well-formed
    When the tool validates the model
    Then none of W070, W071, W072, W073, W074, W076 are emitted

  Scenario: W073 — inner region of a composite substate has no initial
    Given a composite StateDef whose inline-subStates region has no isInitial substate
    When the tool validates the model
    Then a W073 finding is emitted

  Scenario: W076 — transition endpoint does not resolve to a state
    Given a transition whose target names a state that exists nowhere in the machine
    When the tool validates the model
    Then a W076 finding is emitted

  Scenario: draft-suppressed
    Given the unresolved-endpoint machine with status draft
    When the tool validates the model
    Then no W076 finding is emitted

  Scenario: --deny W076 promotes to a gate failure
    Given the unresolved-endpoint machine
    When the tool validates the model with --deny W076
    Then the tool exits non-zero
```

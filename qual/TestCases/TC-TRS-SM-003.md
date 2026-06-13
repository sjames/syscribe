---
id: TC-TRS-SM-003
type: TestCase
testLevel: L3
status: draft
name: "Verify flat state-machine completeness W070–W074: dead/trap/non-determinism/missing-initial/multiple-initial each fire on a crafted defect; a well-formed machine and a parallel/composite machine are clean; draft-suppressed; gateable."
verifies:
  - REQ-TRS-SM-003
---

Verify the single-region state-machine completeness checks against minimal `StateDef`
fixtures, one defect per code plus clean and out-of-scope cases.

```gherkin
Feature: Flat state-machine completeness (W070–W074)

  Scenario: W070 — dead state
    Given a single-region StateDef with a non-initial substate that has no incoming transition
    When the tool validates the model
    Then a W070 finding is emitted

  Scenario: W071 — trap state
    Given a single-region StateDef with a non-final substate that has no outgoing transition
    When the tool validates the model
    Then a W071 finding is emitted

  Scenario: W072 — non-determinism
    Given two unguarded transitions from one source with the same accept payload
    When the tool validates the model
    Then a W072 finding is emitted

  Scenario: W073 — missing initial state
    Given a StateDef with substates but no isInitial substate
    When the tool validates the model
    Then a W073 finding is emitted

  Scenario: W074 — multiple initial states
    Given a StateDef with two isInitial substates
    When the tool validates the model
    Then a W074 finding is emitted

  Scenario: well-formed machine is clean
    Given a connected single-region StateDef with one initial and a final state
    When the tool validates the model
    Then none of W070, W071, W072, W073, W074 are emitted

  Scenario: parallel/composite machine is out of scope
    Given an isParallel StateDef and a StateDef with a composite (typedBy) substate
    When the tool validates the model
    Then none of W070, W071, W072, W073, W074 are emitted

  Scenario: draft-suppressed
    Given the missing-initial machine with status draft
    When the tool validates the model
    Then no W073 finding is emitted

  Scenario: --deny W073 promotes to a gate failure
    Given the missing-initial machine
    When the tool validates the model with --deny W073
    Then the tool exits non-zero
```

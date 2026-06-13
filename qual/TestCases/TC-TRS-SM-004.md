---
id: TC-TRS-SM-004
type: TestCase
testLevel: L3
status: draft
name: "Verify parallel state machines: per-region completeness (region-named W073), cross-region transition W077, parallel arity W078; a well-formed parallel machine is clean; draft-suppressed; gateable."
verifies:
  - REQ-TRS-SM-004
  - REQ-TRS-SM-005
---

Verify the region-aware checks for parallel (orthogonal) state machines.

```gherkin
Feature: Parallel state machine validation (W073 per region, W077, W078)

  Scenario: well-formed parallel machine is clean
    Given an isParallel StateDef with two regions, each with one initial and a connected path
    When the tool validates the model
    Then none of W070, W071, W072, W073, W074, W077, W078 are emitted

  Scenario: W073 — a region has no initial state
    Given an isParallel StateDef where one region has no isInitial substate
    When the tool validates the model
    Then a W073 finding is emitted

  Scenario: W077 — cross-region transition
    Given an isParallel StateDef with a top-level transition connecting two different regions
    When the tool validates the model
    Then a W077 finding is emitted

  Scenario: W078 — parallel state with a single region
    Given an isParallel StateDef with only one region
    When the tool validates the model
    Then a W078 finding is emitted

  Scenario: draft-suppressed
    Given a malformed single-region parallel machine with status draft
    When the tool validates the model
    Then no W078 finding is emitted

  Scenario: --deny W078 promotes to a gate failure
    Given the single-region parallel machine
    When the tool validates the model with --deny W078
    Then the tool exits non-zero
```

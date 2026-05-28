---
type: TestCase
id: TC-SIL-SW-001
title: Software unit test — ConflictChecker conflict matrix formally verified
status: active
testLevel: L3
verifies:
  - REQ-SIL-SW-003
---

```gherkin
Feature: Formal verification of conflict matrix invariants

  Scenario: B-Method proof obligations all discharged
    Given the B-Method abstract machine specification for ConflictChecker
    When Atelier-B is executed on the full specification with all refinements
    Then all proof obligations are discharged automatically or by guided proof
    And zero unproved obligations remain in the proof corpus
    And the proof corpus is archived with a SHA-256 hash for inclusion in the safety case

  Scenario: Model checking confirms no reachable state violates the conflict invariant
    Given the B-Method abstract machine loaded into ProB model checker
    When ProB performs bounded model checking to depth 50 states
    Then no counterexample is found that violates the invariant "no two conflicting routes are simultaneously active"
    And the verification report is signed by the independent safety assessor
```

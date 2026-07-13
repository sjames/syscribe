---
type: TestCase
id: TC-TRS-BL-012
name: "Trace-closure scope seals a goal's connected component and drift-checks it"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/baseline.rs
tags:
  - baseline
  - scope
  - traceability
verifies:
  - REQ-TRS-BL-012
---

Verifies that `frozenScope.closureFrom` restricts the seal to the transitive trace closure of
the seed, excludes unrelated elements, and drift-checks any change within the closure.

```gherkin
Feature: Trace-closure baseline scope

  Scenario: The seal covers only the goal's trace component
    Given a requirement REQ-GOAL with a child REQ-CHILD (derivedFrom) and a verifying TC,
      and an unrelated requirement REQ-OTHER
    When `baseline create --tag REL --frozen-scope "closureFrom=REQ-GOAL"` is run
    Then the manifest includes REQ-GOAL, REQ-CHILD, and the TC
    And it excludes REQ-OTHER

  Scenario: A change within the closure drifts the baseline
    Given a released closure-scoped baseline seeded at REQ-GOAL
    When REQ-CHILD (in the closure) is changed
    Then validation reports E520

  Scenario: A change outside the closure does not drift
    Given the same baseline
    When REQ-OTHER (not in the closure) is changed
    Then no drift is reported for that baseline

  Scenario: An empty closure is refused
    Given `closureFrom` naming only unresolvable seeds
    When create is run
    Then it refuses and writes nothing
```

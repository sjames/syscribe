---
id: TC-TRS-TRACE-011
type: TestCase
testLevel: L3
status: draft
name: "Verify a requirement traced only to a SafetyGoal or CybersecurityGoal is not reported as an orphan (W005)."
verifies:
  - REQ-TRS-TRACE-011
---

Verify that `derivedFromSafetyGoal:` and `derivedFromCybersecurityGoal:` (and
its legacy alias `derivedFromSecurityGoal:`) count as upstream traceability for
the orphan warning `W005` (GH #151).

```gherkin
Feature: goal-derived requirements are not orphans

  Scenario: a requirement derived only from a SafetyGoal is not an orphan
    Given REQ-OR-001 with derivedFromSafetyGoal SG-OR-001 and no derivedFrom
    When the tool validates the model
    Then no W005 finding names REQ-OR-001

  Scenario: a requirement derived only from a CybersecurityGoal is not an orphan
    Given REQ-OR-002 with derivedFromCybersecurityGoal CSG-OR-001
      and REQ-OR-003 with the legacy key derivedFromSecurityGoal CSG-OR-001
    When the tool validates the model
    Then no W005 finding names REQ-OR-002 or REQ-OR-003

  Scenario: a requirement with no upstream link is still an orphan
    Given REQ-OR-004 with no derivedFrom, no goal link and no derived children
    When the tool validates the model
    Then exactly one W005 finding is emitted, naming REQ-OR-004
```

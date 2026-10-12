---
type: TestCase
id: TC-SYS-001
name: isolation
status: active
testLevel: L3
subject: Sys
verifies: [REQ-SYS-001]
---

```gherkin
Feature: isolation
  Where the safe state is m.a == 0 and m.b == 0
  Except under FM-SYS-001: m.b == 0
  Except under FM-SYS-002: m.a == 0

  Scenario: reached at 2.2 s fails within 2 s
    When m receives Slow
    Then within 2 s: m.a == 0 and m.b == 0

  Scenario: reached at 2.2 s passes within 2.5 s
    When m receives Slow
    Then within 2.5 s: m.a == 0 and m.b == 0

  Scenario: reached at 1 s and lost at 1.5 s fails within 2 s
    When m receives Blip
    Then within 2 s: m.a == 0

  Scenario: a transient violation fails always
    When m receives Blip
    And 1100 ms elapses
    Then always m.a == 0
    When 3 s elapses
    Then m.a == 0

  Scenario: a welded a survives degraded
    Given fault FM-SYS-001 is injected
    When m receives Trip
    Then within 1 s the safe state holds

  Scenario: both welded with no clause of their own
    Given fault FM-SYS-001 is injected
    And fault FM-SYS-002 is injected
    When m receives Trip
    Then within 1 s the safe state holds

  @invariant
  Scenario: a trip reaches the safe state and keeps it
    When m receives Trip
    Then within 1 s the safe state holds
    When 2 s elapses
```

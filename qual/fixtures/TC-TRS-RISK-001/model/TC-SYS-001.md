---
type: TestCase
id: TC-SYS-001
name: link invariants
status: active
testLevel: L3
subject: Sys
verifies: [REQ-SYS-001]
---

```gherkin
Feature: link

  @invariant
  Scenario: the receiver never accepts a wrong value
    When tx receives Tick
    Then rx.got == 0 or rx.got == 7
```

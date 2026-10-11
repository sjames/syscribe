---
type: TestCase
id: TC-GTI-COM-003
name: "Event log"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-052
---

```gherkin
Feature: Event log

  Scenario: persistence
    Given 1000 events logged
    When power is cycled
    Then all 1000 events remain with time stamps and fault codes
```

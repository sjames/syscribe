---
type: TestCase
id: TC-GTI-SAF-004
name: "Rapid shutdown (NEC 690.12)"
status: active
testLevel: L5
verifies:
  - REQ-GTI-SYS-033
  - REQ-GTI-SW-005
  - REQ-GTI-HW-005
tags: [reg-unverified]
appliesWhen: Features::Protection::RapidShutdown
---

```gherkin
Feature: Rapid shutdown

  Scenario: AC loss
    Given the array conductors are at 600 V
    When AC is removed
    Then voltage on controlled conductors falls to 30 V within 30 s [unverified]

  Scenario: initiator
    Given the inverter is running
    When the rapid shutdown initiator is operated
    Then the same limit is met
```

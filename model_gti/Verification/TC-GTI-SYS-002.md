---
type: TestCase
id: TC-GTI-SYS-002
name: "Anti-islanding test circuit"
status: active
testLevel: L5
verifies:
  - REQ-GTI-SYS-009
  - REQ-GTI-SW-002
  - REQ-GTI-STK-003
---

```gherkin
Feature: Anti-islanding (IEC 62116 / IEEE 1547.1 style)

  Scenario: island with matched resonant load
    Given a parallel RLC load tuned to the inverter output at 100 % of rated power
    When the grid breaker opens
    Then the inverter stops energising within 2 s
    And both relays are open

  Scenario: island with unbalanced load
    Given the RLC load at 66 % and 133 % of the inverter output
    When the grid breaker opens
    Then the inverter stops energising within 2 s
```

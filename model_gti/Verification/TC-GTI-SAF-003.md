---
type: TestCase
id: TC-GTI-SAF-003
name: "DC arc-fault detection (UL 1699B)"
status: active
testLevel: L5
verifies:
  - REQ-GTI-SYS-032
  - REQ-GTI-SW-004
  - REQ-GTI-HW-004
tags: [reg-unverified]
appliesWhen: Features::Protection::Afci
---

```gherkin
Feature: Series arc

  Scenario: arc generator on a string
    Given an arc generator in series with a string at 700 V and 8 A
    When a series arc is struck
    Then the inverter interrupts the input within the UL 1699B time [unverified]
    And it does not restart until a person resets it

  Scenario: no nuisance trip
    Given normal switching noise and a lightning-like surge
    When the inverter runs for 24 hours
    Then no arc fault is reported
```

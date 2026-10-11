---
type: TestCase
id: TC-GTI-SYS-001
name: "Rated output, power quality and efficiency"
status: active
testLevel: L5
verifies:
  - REQ-GTI-STK-001
  - REQ-GTI-SYS-001
  - REQ-GTI-SYS-002
  - REQ-GTI-SYS-003
  - REQ-GTI-SYS-004
  - REQ-GTI-SYS-014
tags: [reg-unverified]
---

```gherkin
Feature: Rated operation on the grid simulator

  Scenario: rated power at nominal grid
    Given the inverter is connected to a grid simulator at the nominal voltage and frequency of the active profile
    And PV simulators supply 4 kW at 600 V on each of three trackers
    When the inverter reaches steady state
    Then it delivers 10 kVA continuously for 2 hours at 45 °C ambient

  Scenario: current distortion and DC injection
    Given the inverter delivers rated power
    When harmonic and DC content of the grid current are measured
    Then current THD is at most 5 % [unverified]
    And DC current is below the limit of the active grid profile [unverified]

  Scenario: efficiency
    Given the inverter operates at 5, 10, 20, 30, 50, 75 and 100 % load
    When DC-to-AC efficiency is measured at 600 V input
    Then the peak is at least 98.0 % and the weighted value at least 97.5 %
```

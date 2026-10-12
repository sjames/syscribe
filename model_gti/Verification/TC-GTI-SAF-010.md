---
type: TestCase
id: TC-GTI-SAF-010
name: "Current sensing for the perturbation check"
status: active
testLevel: L5
verifies:
  - REQ-GTI-HW-010
tags: [reg-unverified]
---

Bench test of the current sensing the perturbation-response check relies on. The executable chain does not model it (TC-GTI-SAF-009 reads the injected current as commanded).

```gherkin
Feature: Current sensing for the perturbation check

  Scenario: a perturbation of 0.5 % of rated current is resolved
    Given the inverter runs at rated power on a grid simulator
    When a perturbation of 0.5 % of rated current is applied
    Then the measured current step is within the resolution and bandwidth the check needs

  Scenario: the current sensing shares nothing with the voltage chains
    Given both grid-voltage sensing chains are forced to report nominal
    When a perturbation is applied
    Then the current sensing still reports the perturbation
```

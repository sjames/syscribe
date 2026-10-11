---
type: TestCase
id: TC-GTI-ENV-001
name: "Ingress protection and climate"
status: active
testLevel: L5
verifies:
  - REQ-GTI-STK-004
  - REQ-GTI-SYS-040
---

```gherkin
Feature: Environmental tests

  Scenario: IP65
    Given the sealed enclosure
    When the dust and water-jet tests are run
    Then no ingress is found

  Scenario: temperature range
    Given the inverter in a chamber
    When it is operated from −25 °C to +60 °C
    Then it starts and runs, derated above 45 °C
```

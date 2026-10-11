---
type: TestCase
id: TC-GTI-PLAT-001
name: "Region selection and commissioning"
status: active
testLevel: L4
verifies:
  - REQ-GTI-STK-006
  - REQ-GTI-STK-008
  - REQ-GTI-SYS-060
  - REQ-GTI-SYS-080
---

```gherkin
Feature: Commissioning

  Scenario: profile selection
    Given a new inverter
    When an installer selects the region with a commissioning credential
    Then the grid profile of the region is active and shown locally

  Scenario: profile lock
    Given a commissioned inverter
    When a region change is attempted without a service credential
    Then it is refused

  Scenario: self-check
    Given a commissioned inverter
    When commissioning completes
    Then insulation, grid presence, profile and communication results are shown locally without a network
```

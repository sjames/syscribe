---
type: TestCase
id: TC-UAV-SAFETY-001
name: "System safety campaign: no injury or damage across nominal, contingency, and loss-of-control phases"
status: active
testLevel: L5
verifies:
  - REQ-UAV-SAFE-000
tags:
  - safety
  - integration
  - flight-test
---

System-level hardware-in-the-loop safety campaign demonstrating the emergent safety goal across
all flight phases — that combined sensor-fault detection, autonomous safe landing, and link-loss
handling keep the UAV from causing injury to persons or damage to property. Composes the derived
safety behaviours into one end-to-end safety verification (ASIL B), complementing the
parent's analysis-based argument.

Run: `cargo xtask hil -- safety-campaign --all-phases`

```gherkin
Feature: System safety integration

  Background:
    Given the UAV is assembled in a flight-ready configuration on the HIL safety bench
    And the safety monitoring and fault-injection interfaces are active

  Scenario: Nominal flight causes no hazardous condition
    When a full nominal mission is flown
    Then no hazardous event is recorded and the UAV lands safely

  Scenario: Single sensor failure is contained without loss of control
    When a single sensor failure is injected in flight
    Then the flight controller detects the fault and maintains a safe attitude
    And the UAV transitions to a degraded-but-safe mode

  Scenario: Command-link loss triggers a safe recovery
    When the command link is lost mid-mission
    Then the UAV performs autonomous return-to-home or a controlled safe landing
    And it does not enter an uncontrolled descent over a populated area
```

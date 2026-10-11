---
type: TestCase
id: TC-GTI-COM-001
name: "SunSpec Modbus interface"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-050
appliesWhen: Features::Comms::SunSpecModbus
---

```gherkin
Feature: Modbus

  Scenario: model discovery and control
    Given a SunSpec client
    When it reads the model chain and writes a permitted setting
    Then values match the inverter and the setting takes effect
```

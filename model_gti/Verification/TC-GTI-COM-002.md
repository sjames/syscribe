---
type: TestCase
id: TC-GTI-COM-002
name: "IEEE 2030.5 client"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-051
appliesWhen: Features::Comms::Ieee2030_5
---

```gherkin
Feature: IEEE 2030.5

  Scenario: utility server interaction
    Given a conformance test server
    When the inverter registers and receives a DER control
    Then it applies the control and reports status
```

---
type: TestCase
id: TC-GTI-GRID-005
name: "Remote curtailment"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-013
appliesWhen: Features::GridSupport::RemoteCurtailment
---

```gherkin
Feature: Curtailment command

  Scenario: authenticated command
    Given the inverter at rated power
    When an authenticated command limits power to 50 %
    Then active power is at 50 % within 10 s and stays there

  Scenario: unauthenticated command
    Given the inverter at rated power
    When an unauthenticated command arrives
    Then it is rejected and logged
```

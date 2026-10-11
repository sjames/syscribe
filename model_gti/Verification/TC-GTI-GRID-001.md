---
type: TestCase
id: TC-GTI-GRID-001
name: "US reactive power capability"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-005
tags: [reg-unverified]
appliesWhen: Features::Region::US
---

```gherkin
Feature: Power factor on the grid simulator

  Scenario: 0.8 leading and lagging
    Given the inverter at rated apparent power
    When the power factor is set to 0.8 leading and then 0.8 lagging
    Then it holds each setting within 0.02
```

---
type: TestCase
id: TC-GTI-GRID-002
name: "India power-factor capability"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-006
tags: [reg-unverified]
appliesWhen: Features::Region::IN
---

```gherkin
Feature: Power factor on the grid simulator

  Scenario: 0.95 leading and lagging above 20 % load
    Given the inverter above 20 % load
    When the power factor is set to 0.95 leading and then 0.95 lagging
    Then it holds each setting within 0.02
```

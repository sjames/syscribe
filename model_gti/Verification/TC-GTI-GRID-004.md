---
type: TestCase
id: TC-GTI-GRID-004
name: "Frequency-watt"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-012
appliesWhen: Features::GridSupport::FreqWatt
---

```gherkin
Feature: Frequency-watt curve

  Scenario: power is reduced as frequency rises
    Given frequency-watt is enabled
    When frequency rises above the dead band
    Then active power falls along the profile slope
```

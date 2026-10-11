---
type: TestCase
id: TC-GTI-ENV-003
name: "India EMC (IEC 61000-6)"
status: active
testLevel: L5
verifies:
  - REQ-GTI-SYS-043
appliesWhen: Features::Region::IN
---

```gherkin
Feature: Emissions and immunity

  Scenario: emissions and immunity
    Given the inverter at rated power
    When emissions and immunity are tested
    Then the IEC 61000-6 levels required by BIS CRS are met
```

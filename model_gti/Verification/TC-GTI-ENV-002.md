---
type: TestCase
id: TC-GTI-ENV-002
name: "US EMC (FCC Part 15)"
status: active
testLevel: L5
verifies:
  - REQ-GTI-SYS-042
appliesWhen: Features::Region::US
---

```gherkin
Feature: Emissions

  Scenario: radiated and conducted emissions
    Given the inverter at rated power
    When emissions are measured
    Then they are below the FCC Part 15 limits of the class
```

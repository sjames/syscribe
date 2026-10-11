---
type: TestCase
id: TC-GTI-GRID-003
name: "Volt-var"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-011
appliesWhen: Features::GridSupport::VoltVar
---

```gherkin
Feature: Volt-var curve

  Scenario: curve is followed
    Given volt-var is enabled with the profile curve
    When grid voltage is stepped through the curve points
    Then reactive power follows the curve within the accuracy of the profile
```

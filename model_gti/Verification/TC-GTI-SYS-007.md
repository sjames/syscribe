---
type: TestCase
id: TC-GTI-SYS-007
name: "String monitoring"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-024
appliesWhen: Features::Mppt::StringMonitoring
---

```gherkin
Feature: Under-performing string

  Scenario: string loses output
    Given two strings on one tracker produce equal current
    When one string drops to 50 % current for more than 15 minutes in good irradiance
    Then the inverter raises a string warning naming the string
```

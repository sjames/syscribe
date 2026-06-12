---
id: TC-V4-WDT-001
type: TestCase
testLevel: L3
status: approved
name: "Test TC-V4-WDT-001"
verifies:
  - REQ-V4-WDT-001
appliesWhen: Features::Wdt
---

```gherkin
Feature: TC-V4-WDT-001
  Scenario: nominal
    Given the system
    Then REQ-V4-WDT-001 is satisfied
```

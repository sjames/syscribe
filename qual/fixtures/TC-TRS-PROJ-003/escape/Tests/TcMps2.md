---
id: TC-PROJ3-001
type: TestCase
testLevel: L3
status: active
name: "TC-PROJ3-001"
verifies:
  - REQ-PROJ3-WDT-001
appliesWhen: Features::Mps2
---

```gherkin
Feature: TC-PROJ3-001
  Scenario: s
    Given x
    Then REQ-PROJ3-WDT-001 holds
```

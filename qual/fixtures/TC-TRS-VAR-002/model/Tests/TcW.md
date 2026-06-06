---
id: TC-V2-WDT-001
type: TestCase
testLevel: L3
status: approved
title: "Test TC-V2-WDT-001"
verifies:
  - REQ-V2-WDT-001
appliesWhen: Features::Wdt
---

```gherkin
Feature: TC-V2-WDT-001
  Scenario: nominal
    Given the system
    Then REQ-V2-WDT-001 is satisfied
```

---
id: TC-RDY-WDT-001
type: TestCase
testLevel: L3
status: active
title: "Test TC-RDY-WDT-001"
verifies:
  - REQ-RDY-WDT-001
appliesWhen: Features::Wdt
---

```gherkin
Feature: TC-RDY-WDT-001
  Scenario: nominal
    Given the system
    Then REQ-RDY-WDT-001 is satisfied
```

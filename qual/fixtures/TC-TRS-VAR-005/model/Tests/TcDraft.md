---
id: TC-V5-WDT-004
type: TestCase
testLevel: L3
status: draft
title: "Test TC-V5-WDT-004"
verifies:
  - REQ-V5-WDT-004
appliesWhen: Features::Wdt
---

```gherkin
Feature: TC-V5-WDT-004
  Scenario: nominal
    Given the system
    Then REQ-V5-WDT-004 is satisfied
```

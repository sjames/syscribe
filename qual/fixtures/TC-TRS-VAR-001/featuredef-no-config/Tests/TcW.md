---
id: TC-VAR1-003
type: TestCase
testLevel: L3
status: approved
title: "Test TC-VAR1-003"
verifies:
  - REQ-VAR1-003
appliesWhen: Features::Wdt
---

```gherkin
Feature: TC-VAR1-003
  Scenario: nominal
    Given the system
    Then REQ-VAR1-003 is satisfied
```

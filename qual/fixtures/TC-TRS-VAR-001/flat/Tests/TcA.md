---
id: TC-VAR1-001
type: TestCase
testLevel: L3
status: approved
title: "Test TC-VAR1-001"
verifies:
  - REQ-VAR1-001
---

```gherkin
Feature: TC-VAR1-001
  Scenario: nominal
    Given the system
    Then REQ-VAR1-001 is satisfied
```

---
id: TC-FUNC-001
type: TestCase
title: "Functional check of the control loop (not a timing measurement)"
status: active
testLevel: L3
verifies:
  - REQ-WCET-001
---
```gherkin
Feature: control loop functional
  Scenario: nominal
    Given nominal inputs
    Then the loop produces correct output
```

---
id: TC-OUT9-FAIL-001
type: TestCase
testLevel: L3
status: approved
title: "Test for the failing requirement"
verifies:
  - REQ-OUT9-FAIL-001
testFunctions:
  - function: "widget::tests::fail_boom"
    scenario: "It fails"
---

```gherkin
Feature: TC-OUT9-FAIL-001
  Scenario: nominal
    Given the system
    Then REQ-OUT9-FAIL-001 is satisfied
```

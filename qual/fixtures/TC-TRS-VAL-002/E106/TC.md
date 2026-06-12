---
id: TC-TST-E106-001
type: TestCase
name: "TestCase with mismatched testFunctions scenario"
status: draft
testLevel: L3
verifies:
  - REQ-TST-E106-001
testFunctions:
  - function: "my_tests::tests::nonexistent_scenario"
    scenario: "This scenario title does not exist in the Gherkin body"
---

TestCase with a `testFunctions:` entry whose `scenario:` string does not match any `Scenario:` or `Scenario Outline:` title in the Gherkin blocks — should produce E106.

```gherkin
Feature: E106 trigger fixture

  Scenario: Actual scenario title in the file
    Given the system is configured
    When the action is performed
    Then the expected result occurs
```

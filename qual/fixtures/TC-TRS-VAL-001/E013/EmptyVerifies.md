---
id: TC-TST-001
type: TestCase
name: x
status: draft
testLevel: L3
verifies: []
---

TestCase with an empty `verifies` list — should produce E013.

```gherkin
Feature: Test fixture

  Scenario: Baseline
    Given the system is configured
    When the action is performed
    Then the expected result occurs
```

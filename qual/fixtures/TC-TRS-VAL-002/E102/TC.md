---
id: TC-TST-001
type: TestCase
name: x
status: draft
testLevel: L3
verifies:
  - REQ-NONEXISTENT-001
---

TestCase that references a requirement id that does not exist in the model — should produce E102.

```gherkin
Feature: Test fixture

  Scenario: Baseline
    Given the system is configured
    When the action is performed
    Then the expected result occurs
```

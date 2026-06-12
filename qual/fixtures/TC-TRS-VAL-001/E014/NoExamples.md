---
id: TC-TST-001
type: TestCase
name: x
status: draft
testLevel: L3
verifies:
  - REQ-TST-001
---

TestCase with a `Scenario Outline:` but no `Examples:` table — should produce E014.

```gherkin
Feature: Test fixture

  Scenario Outline: Parameterised test without examples
    Given the system is configured with <param>
    When the action is performed
    Then the expected result occurs
```

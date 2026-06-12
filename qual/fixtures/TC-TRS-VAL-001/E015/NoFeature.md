---
id: TC-TST-001
type: TestCase
name: x
status: draft
testLevel: L3
verifies:
  - REQ-TST-001
---

TestCase with a gherkin block that starts with `Scenario:` instead of `Feature:` — should produce E015.

```gherkin
  Scenario: Missing feature header
    Given the system is configured
    When the action is performed
    Then the expected result occurs
```

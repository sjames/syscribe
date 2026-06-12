---
id: TC-TST-001
type: TestCase
name: x
status: draft
testLevel: L6
verifies:
  - REQ-TST-001
---

TestCase with invalid `testLevel: L6` (only L1–L5 are valid) — should produce E008.

```gherkin
Feature: Test fixture

  Scenario: Baseline
    Given the system is configured
    When the action is performed
    Then the expected result occurs
```

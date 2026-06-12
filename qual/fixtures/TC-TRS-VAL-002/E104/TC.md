---
id: TC-TST-001
type: TestCase
name: x
status: draft
testLevel: L3
verifies:
  - MyPart
---

TestCase with `verifies:` pointing to a PartDef (not a Requirement) — should produce E104.

```gherkin
Feature: Test fixture

  Scenario: Baseline
    Given the system is configured
    When the action is performed
    Then the expected result occurs
```

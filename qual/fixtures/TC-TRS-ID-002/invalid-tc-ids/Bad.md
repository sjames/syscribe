---
id: TC-001
type: TestCase
name: Invalid TC id (too short)
status: draft
testLevel: L3
verifies:
  - REQ-TST-XRF-001
---

Test case with an invalid id `TC-001` — only one segment after the prefix, missing the required second segment. Should produce E008 or equivalent TC id error.

```gherkin
Feature: Test fixture

  Scenario: Baseline
    Given the system is configured
    When the action is performed
    Then the expected result occurs
```

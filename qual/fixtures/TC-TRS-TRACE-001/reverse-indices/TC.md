---
id: TC-TRC-001
type: TestCase
name: Test case verifying REQ-TRC-001
status: draft
testLevel: L3
verifies:
  - REQ-TRC-001
---

Test case that references `REQ-TRC-001` — the reverse `verifiedBy` index on the requirement should be populated.

```gherkin
Feature: Test fixture

  Scenario: Baseline
    Given the system is configured
    When the action is performed
    Then the expected result occurs
```

---
id: TC-TRS-BT-026
type: TestCase
testLevel: L3
status: active
name: "Verify fallback is validated and exported"
verifies:
  - REQ-TRS-BT-026
---

```gherkin
Feature: Parameter fallback
  Scenario: out of range
    Given a fallback outside range
    When validate is run
    Then an error is raised

  Scenario: exported
    Given a valid fallback
    When params export is run
    Then it appears as a comment
```

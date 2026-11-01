---
id: TC-TRS-BT-034
type: TestCase
testLevel: L3
status: active
name: "Verify late-bound variability reaching a safety requirement needs covering verification"
verifies:
  - REQ-TRS-BT-034
---

```gherkin
Feature: Late-bound safety rule
  Scenario: uncovered
    Given a runtime parameter placed in an ASIL requirement's text with no covering test
    When validate is run
    Then the rule flags it, and --deny makes it fail

  Scenario: covered
    Given a TestCase covering the parameter's range
    When validate is run
    Then the flag clears
```

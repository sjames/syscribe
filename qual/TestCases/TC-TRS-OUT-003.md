---
id: TC-TRS-OUT-003
type: TestCase
testLevel: L3
status: draft
name: "Verify that the report includes a summary section with error and warning counts."
verifies:
  - REQ-TRS-OUT-003
---

Verify that the report includes a summary section with error and warning counts.

```gherkin
Feature: Summary count in report

  Scenario: Summary counts match the finding list
    Given a model with exactly 3 errors and 2 warnings
    When the report is examined
    Then the summary section shows Errors: 3
    And the summary section shows Warnings: 2
    And the number of E-code rows in the findings table is 3
    And the number of W-code rows in the findings table is 2

  Scenario: Clean model summary shows zero counts
    Given a model with no errors and no warnings
    When the report is examined
    Then the summary shows Errors: 0 and Warnings: 0
```

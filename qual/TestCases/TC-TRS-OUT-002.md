---
id: TC-TRS-OUT-002
type: TestCase
testLevel: L3
status: draft
name: "Verify that each finding in the report contains severity, code, element reference, and description."
verifies:
  - REQ-TRS-OUT-002
---

Verify that each finding in the report contains severity, code, element reference, and description.

```gherkin
Feature: Finding detail in report

  Scenario: Each finding line is parseable to four fields
    Given a model that produces at least one Error and one Warning finding
    When the report is captured and each finding line is parsed
    Then every finding line contains a severity indicator (Error or Warning)
    And a rule code matching E-nnn or W-nnn
    And an element reference (qualified name, id, or file path)
    And a human-readable description

  Scenario: Finding table columns are consistently labelled
    Given any model that produces findings
    When the report is examined
    Then the findings table has columns for Code, File/Element, and Message
```

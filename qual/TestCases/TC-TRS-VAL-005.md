---
id: TC-TRS-VAL-005
type: TestCase
testLevel: L3
status: draft
name: "Verify that each finding includes the required fields: rule code, element reference, and description."
verifies:
  - REQ-TRS-VAL-005
  - REQ-TRS-VAL-006
---

Verify that each finding includes the required fields: rule code, element reference, and description.

```gherkin
Feature: Finding content and attribution

  Scenario: Parse-time finding is attributed to the source file
    Given a file model/A/B.md with invalid YAML frontmatter
    When the tool is invoked
    Then the E002 finding references model/A/B.md or its qualified name

  Scenario: Model-time finding includes the offending element's reference
    Given an element with a dangling verifies: reference
    When the tool is invoked
    Then the E102 finding includes the element's qualified name or id

  Scenario: Each finding output line contains a code, a reference, and a description
    Given any model that produces at least one finding
    When the tool output is parsed
    Then each finding line contains a recognisable rule code (E-nnn or W-nnn)
    And an element identifier (qualified name or file path)
    And a human-readable description of the violation
```

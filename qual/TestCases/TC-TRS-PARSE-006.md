---
id: TC-TRS-PARSE-006
type: TestCase
testLevel: L3
status: draft
name: "Verify that a file with unparseable frontmatter produces a warning but does not halt processing."
verifies:
  - REQ-TRS-PARSE-006
---

Verify that a file with unparseable frontmatter produces a warning but does not halt processing.

```gherkin
Feature: Unparseable frontmatter is non-fatal

  Scenario: Malformed YAML frontmatter produces a warning
    Given a model with one valid element and one file with malformed YAML frontmatter
    When the tool is invoked
    Then a warning or error finding is emitted for the malformed file
    And the valid element is still reported in the element count

  Scenario: File with no frontmatter produces a warning
    Given a model with one valid element and one plain Markdown file with no frontmatter
    When the tool is invoked
    Then a warning is emitted for the file with no frontmatter
    And the valid element count is unaffected
```

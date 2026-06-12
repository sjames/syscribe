---
id: TC-TRS-PARSE-007
type: TestCase
testLevel: L3
status: draft
name: "Verify that frontmatter is recognized only when the opening --- is the first line."
verifies:
  - REQ-TRS-PARSE-007
---

Verify that frontmatter is recognized only when the opening --- is the first line.

```gherkin
Feature: Frontmatter delimiter on first line

  Scenario: File with --- on first line is parsed correctly
    Given a .md file whose very first line is ---
    And valid YAML frontmatter between --- delimiters
    When the tool is invoked
    Then the element is loaded without parse errors

  Scenario: File with a blank first line is treated as missing frontmatter
    Given a .md file with a blank line before the opening ---
    When the tool is invoked
    Then the file is treated as having no frontmatter
    And a warning finding is emitted for that file
```

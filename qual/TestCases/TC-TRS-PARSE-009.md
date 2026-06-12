---
id: TC-TRS-PARSE-009
type: TestCase
testLevel: L3
status: draft
name: "Verify that a file without a type: field is skipped with a warning."
verifies:
  - REQ-TRS-PARSE-009
---

Verify that a file without a type: field is skipped with a warning.

```gherkin
Feature: Missing type: field handling

  Scenario: File with no type: field is skipped with a warning
    Given a .md file with valid YAML frontmatter that omits the type: field
    When the tool is invoked
    Then a warning finding is emitted for that file
    And the file is not counted as a model element
    And no E004 finding is emitted (type: is treated specially, not as a required field)

  Scenario: File with type: present is processed normally
    Given a .md file with valid YAML frontmatter including type: PartDef
    When the tool is invoked
    Then the file is processed as a PartDef element
    And no warning about missing type: is emitted
```

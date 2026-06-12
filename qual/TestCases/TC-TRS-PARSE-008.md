---
id: TC-TRS-PARSE-008
type: TestCase
testLevel: L3
status: draft
name: "Verify that invalid YAML frontmatter produces error E002."
verifies:
  - REQ-TRS-PARSE-008
---

Verify that invalid YAML frontmatter produces error E002.

```gherkin
Feature: YAML 1.2 parsing

  Scenario: Valid YAML frontmatter is parsed without error
    Given a .md file with syntactically correct YAML 1.2 frontmatter
    When the tool is invoked
    Then no E002 finding is emitted for that file

  Scenario: Invalid YAML frontmatter produces E002
    Given a .md file with frontmatter containing a YAML syntax error (e.g. unbalanced braces)
    When the tool is invoked
    Then exactly one E002 finding is emitted referencing that file
```

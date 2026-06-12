---
id: TC-TRS-ELEM-002
type: TestCase
testLevel: L3
status: draft
name: "Verify that an unrecognised type: value produces exactly one E005 finding."
verifies:
  - REQ-TRS-ELEM-002
---

Verify that an unrecognised type: value produces exactly one E005 finding.

```gherkin
Feature: Unknown type: produces E005

  Scenario: Completely unknown type value produces E005
    Given a .md file with type: BogusWidget in its frontmatter
    When the tool is invoked
    Then exactly one E005 finding is emitted for that file
    And no other type-related error is emitted for the same file

  Scenario: Misspelled known type produces E005
    Given a .md file with type: Partdef (wrong case) in its frontmatter
    When the tool is invoked
    Then an E005 finding is emitted (type values are case-sensitive)
```

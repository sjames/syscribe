---
id: TC-TRS-CFLD-002
type: TestCase
testLevel: L3
status: draft
name: "Verify the --where custom-field query: exact, regex, list-membership, presence, and bad-predicate exit."
verifies:
  - REQ-TRS-CFLD-002
---

```gherkin
Feature: --where custom-field query
  Scenario: operators
    Given a model with elements carrying custom_fields
    When ls --where is invoked with custom.key=, =~, ~= and bare-presence forms
    Then each filters the elements correctly
    And an unparseable predicate exits non-zero
  Scenario: unsupported operators are usage errors (issue #129)
    Given the same model
    When list, ls or find is invoked with --where using !=, ==, ~, >, <, >= or <=
    Then the command exits 1 with nothing on stdout
    And stderr names the operator and lists the supported forms (=, =~, ~=, presence)
    And the documented list PartDef --where and find . --where forms still work
```

---
id: TC-TRS-VAL-008
type: TestCase
testLevel: L3
status: draft
title: "Verify that safety-level, standards-compliance, and type-field validation rules are enforced."
verifies:
  - REQ-TRS-VAL-008
---

Verify that each safety-level, standards-compliance, and no-type-field validation rule is triggered by its corresponding crafted model condition.

```gherkin
Feature: Safety and standards validation rule enforcement

  Scenario Outline: Each error code is produced by its trigger condition
    Given a model file that satisfies the trigger condition for <code>
    When the tool is invoked
    Then at least one <code> finding is emitted

    Examples:
      | code  | trigger condition                                                             |
      | E019  | dalLevel: set to a value not in A, B, C, D, E                                |
      | E020  | verificationMethod: set to a value not in the allowed enum                   |
      | E021  | coverageTarget: set to a value not in statement, branch, MCDC                |
      | E022  | requirementKind: set to a value not in the allowed enum                      |

  Scenario: W008 is emitted for a file with valid frontmatter but no type field
    Given a .md file with valid YAML frontmatter containing no type: key
    When the tool is invoked
    Then at least one W008 finding is emitted with severity Warning

  Scenario: W701 is emitted for a Requirement with high ASIL but no verificationMethod
    Given a Requirement with asilLevel: B and no verificationMethod: field
    When the tool is invoked
    Then at least one W701 finding is emitted with severity Warning

  Scenario: W702 is emitted for an ASIL-D Requirement with no L5 TestCase
    Given a Requirement with asilLevel: D and status: approved
    And an active TestCase that verifies it at testLevel: L3
    And no active TestCase at testLevel: L5 verifies it
    When the tool is invoked
    Then at least one W702 finding is emitted with severity Warning

  Scenario: W703 is emitted when both asilLevel and dalLevel are set
    Given an element with both asilLevel: and dalLevel: fields set
    When the tool is invoked
    Then at least one W703 finding is emitted with severity Warning
```

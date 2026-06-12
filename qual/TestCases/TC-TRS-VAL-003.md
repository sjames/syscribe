---
id: TC-TRS-VAL-003
type: TestCase
testLevel: L3
status: draft
name: "Verify that each warning rule is triggered by its condition with Warning severity."
verifies:
  - REQ-TRS-VAL-003
---

Verify that each warning rule is triggered by its condition with Warning severity.

```gherkin
Feature: Warning rule enforcement

  Scenario Outline: Each warning code is produced by its trigger condition
    Given a model that satisfies the trigger condition for <code>
    When the tool is invoked
    Then at least one <code> finding is emitted with severity Warning

    Examples:
      | code  | trigger condition                                                              |
      | W001  | Requirement normative text contains no "shall"                                |
      | W002  | approved/implemented Requirement has no active TestCase in verifiedBy:        |
      | W003  | verified Requirement has empty or all-retired verifiedBy:                     |
      | W004  | sourceFile: path does not exist on disk                                        |
      | W005  | Requirement has neither derivedFrom: nor derivedChildren                       |
      | W006  | both silLevel: and asilLevel: set on the same element                          |
      | W007  | frontmatter contains an unrecognised key (lenient mode)                        |
      | W300  | leaf Requirement at approved/implemented has no satisfying element             |
      | W301  | leaf Requirement satisfied by more than one element                            |
      | W302  | leaf Requirement at implemented/verified has reqDomain: system                 |
      | W303  | breakdownAdr: references a proposed ADR while Requirement is approved+         |
      | W304  | isDeploymentPackage: true combined with domain: hardware                       |
      | W305  | parent Requirement at approved+ has no system-level TestCase                   |
```

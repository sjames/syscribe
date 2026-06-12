---
id: TC-TRS-VAL-001
type: TestCase
testLevel: L3
status: draft
name: "Verify that each parse-time error rule is triggered by the corresponding malformed input."
verifies:
  - REQ-TRS-VAL-001
---

Verify that each parse-time error rule is triggered by the corresponding malformed input.

```gherkin
Feature: Parse-time error rule enforcement

  Scenario Outline: Each parse-time error code is produced by its trigger condition
    Given a model file that satisfies the trigger condition for <code>
    When the tool is invoked
    Then exactly one <code> finding is emitted for that file

    Examples:
      | code  | trigger condition                                                   |
      | E001  | file does not begin with ---                                       |
      | E002  | frontmatter contains invalid YAML 1.2                              |
      | E004  | a required field for the element type is absent                    |
      | E005  | type: value is not in the element type inventory                   |
      | E006  | id: is present but does not match the required pattern             |
      | E007  | status: value is not in the allowed enum for the element type      |
      | E008  | testLevel: value is not in L1–L5                                   |
      | E009  | silLevel: value is not an integer in 1–4                           |
      | E010  | asilLevel: value is not in A–D                                     |
      | E011  | TestCase body has no gherkin fenced block                          |
      | E012  | Requirement body has no normative text                             |
      | E013  | verifies: list is present but empty                                |
      | E014  | Scenario Outline: block has no Examples: table                     |
      | E015  | first Gherkin block has no Feature: line                           |
      | E300  | ADR.id does not match ADR-* pattern                                |
      | E301  | ADR missing required field (id, title, or status)                  |
      | E302  | reqDomain: value not in system/hardware/software                   |
      | E303  | domain: value not in system/hardware/software                      |
      | E304  | ADR.status value not in allowed enum                               |

  Scenario: E003 is emitted in strict mode for unrecognised frontmatter key
    Given a model file with an unrecognised key in strict parsing mode
    When the tool is invoked in strict mode
    Then an E003 finding is emitted for the unrecognised key
```

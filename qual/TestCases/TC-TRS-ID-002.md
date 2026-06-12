---
id: TC-TRS-ID-002
type: TestCase
testLevel: L3
status: draft
name: "Verify that TestCase elements are validated against the TC-* id pattern."
verifies:
  - REQ-TRS-ID-002
---

Verify that TestCase elements are validated against the TC-* id pattern.

```gherkin
Feature: TestCase id pattern validation

  Scenario Outline: Valid TC-* id patterns are accepted
    Given a TestCase element with id: <valid_id>
    When the tool is invoked
    Then no E006 finding is emitted for that element

    Examples:
      | valid_id                |
      | TC-TRS-PARSE-001        |
      | TC-SCHED-BITMAP-001     |
      | TC-SYNC-SEM-002         |

  Scenario Outline: Invalid TC-* id patterns produce E006
    Given a TestCase element with id: <invalid_id>
    When the tool is invoked
    Then an E006 finding is emitted for that element

    Examples:
      | invalid_id       |
      | tc-TRS-001       |
      | TC001            |
      | TC-A-001         |
```

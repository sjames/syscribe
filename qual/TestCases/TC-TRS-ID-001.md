---
id: TC-TRS-ID-001
type: TestCase
testLevel: L3
status: draft
name: "Verify that Requirement elements are validated against the REQ-* id pattern."
verifies:
  - REQ-TRS-ID-001
---

Verify that Requirement elements are validated against the REQ-* id pattern.

```gherkin
Feature: Requirement id pattern validation

  Scenario Outline: Valid REQ-* id patterns are accepted
    Given a Requirement element with id: <valid_id>
    When the tool is invoked
    Then no E006 finding is emitted for that element

    Examples:
      | valid_id              |
      | REQ-TRS-001           |
      | REQ-SCHED-BITMAP-001  |
      | REQ-PORT-CTX-001      |
      | REQ-AB-999            |

  Scenario Outline: Invalid REQ-* id patterns produce E006
    Given a Requirement element with id: <invalid_id>
    When the tool is invoked
    Then an E006 finding is emitted for that element

    Examples:
      | invalid_id       |
      | REQ-trs-001      |
      | REQTRS001        |
      | REQ-001          |
      | REQ-A-001        |
```

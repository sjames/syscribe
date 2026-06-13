---
id: TC-TRS-OUT-015
type: TestCase
testLevel: L3
status: draft
name: "Verify list AssumptionOfUse emits SRAC-oriented table columns and appliesTo/body in JSON"
verifies:
  - REQ-TRS-OUT-015
---

```gherkin
Feature: list AssumptionOfUse specialized output

  Scenario: list AssumptionOfUse prints SRAC-oriented table columns
    Given a model containing an AssumptionOfUse element AOU-SYS-001 with appliesTo: [SG-X, SG-Y] and status: active
    When the user runs list AssumptionOfUse
    Then the output table contains the headers ID, Name, Applies To, Status
    And the row for AOU-SYS-001 shows SG-X, SG-Y in the Applies To column

  Scenario: AOU with empty appliesTo shows em-dash in Applies To column
    Given an AssumptionOfUse element AOU-SYS-002 with no appliesTo field
    When the user runs list AssumptionOfUse
    Then the Applies To cell for AOU-SYS-002 shows a dash or em-dash

  Scenario: list AssumptionOfUse --json includes appliesTo array and body string
    Given an AssumptionOfUse AOU-SYS-001 with appliesTo: [SG-X] and a non-empty Markdown body
    When the user runs list AssumptionOfUse --json
    Then the JSON entry for AOU-SYS-001 contains "appliesTo": ["SG-X"]
    And the JSON entry contains a non-null "body" string

  Scenario: list AssumptionOfUse --json sets body null for empty body
    Given an AssumptionOfUse AOU-SYS-003 with no body text
    When the user runs list AssumptionOfUse --json
    Then the JSON entry for AOU-SYS-003 contains "body": null

  Scenario: list Requirement still uses generic table columns (no regression)
    Given a model with Requirement elements
    When the user runs list Requirement
    Then the output does not show Applies To or body columns
```

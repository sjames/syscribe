---
id: TC-TRS-VAL-016
type: TestCase
testLevel: L3
status: draft
name: "Verify wcet queryability (--has-wcet, list --json) and the W029 WCET-not-measured check."
verifies:
  - REQ-TRS-VAL-016
---

Verify that `wcet` is filterable/queryable and that an unbacked timing claim on a SIL/ASIL requirement is flagged W029.

```gherkin
Feature: WCET claim evidence and queryability

  Scenario: list --has-wcet keeps only requirements with a wcet
    Given a model with some requirements declaring wcet and some not
    When the tool runs `list Requirement --has-wcet`
    Then only the wcet-bearing requirements are listed

  Scenario: list --json includes wcet
    When the tool runs `list Requirement --has-wcet --json`
    Then each emitted object carries its wcet value

  Scenario: a SIL requirement with wcet and no measuring test produces W029
    Given a non-draft silLevel requirement with wcet and only an L3 functional test
    When the tool validates the model
    Then a W029 finding is emitted

  Scenario: a measuring test clears W029
    Given the same requirement also verified by an active L5 (or timing-tagged) TestCase
    When the tool validates the model
    Then no W029 finding is emitted

  Scenario: a requirement without wcet or integrity level produces no W029
    Given requirements lacking wcet or lacking silLevel/asilLevel
    When the tool validates the model
    Then no W029 finding is emitted for them

  Scenario: W029 is gateable
    Given a model with a W029
    When the tool validates with --deny W029
    Then the tool exits non-zero
```

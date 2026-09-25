---
id: TC-TRS-SM-007
type: TestCase
testLevel: L3
status: draft
name: "Verify W929 fires for a top-level transition without source and a transition without target, not for a nested transition with an implicit source; draft-suppressed; gateable."
verifies:
  - REQ-TRS-SM-009
---

```gherkin
Feature: Incomplete state-machine transitions are reported (TC-TRS-SM-007)

  Scenario: a top-level transition without source raises W929
    Given a StateDef whose top-level transitions: list has an entry with a target but no source
    When the tool validates the model
    Then W929 names the missing source

  Scenario: a transition without target raises W929
    Given a substate whose nested transition has no target
    When the tool validates the model
    Then W929 names the missing target

  Scenario: a nested transition with an implicit source raises no W929
    Given a well-formed machine whose nested transitions omit source
    When the tool validates the model
    Then no W929 is reported

  Scenario: W929 is draft-suppressed and gateable
    Given the incomplete machine marked status: draft
    When the tool validates the model
    Then no W929 is reported
    And validating the non-draft machine with --deny W929 exits non-zero
```

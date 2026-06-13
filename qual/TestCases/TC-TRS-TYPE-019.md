---
id: TC-TRS-TYPE-019
type: TestCase
testLevel: L3
status: draft
name: "Verify the native TradeStudy element: E869–E877 structural rules, W061–W064 advisories, normalised/weighted/ranked scoring, and the trade-study list/detail commands plus template TradeStudy."
verifies:
  - REQ-TRS-TYPE-019
---

Verify recognition, validation, scoring, and the CLI surface for `TradeStudy`.

```gherkin
Feature: Native TradeStudy element (§15)

  Scenario: well-formed TradeStudy validates clean
    Given a complete TradeStudy with criteria, alternatives, a full score matrix, objective and decision
    When the tool validates the model
    Then none of E869–E877 or W061–W064 are emitted

  Scenario: E869 — missing required field
    Given a TradeStudy missing scores
    When the tool validates the model
    Then an E869 finding is emitted

  Scenario: E870 — bad id pattern
    Given a TradeStudy whose id is not a TRD-* id
    When the tool validates the model
    Then an E870 finding is emitted

  Scenario: E871 — criterion missing a field
    Given a criteria entry missing direction
    When the tool validates the model
    Then an E871 finding is emitted

  Scenario: E872 — weight out of range
    Given a criterion weight of 2.0
    When the tool validates the model
    Then an E872 finding is emitted

  Scenario: E873 — bad direction
    Given a criterion direction that is neither maximize nor minimize
    When the tool validates the model
    Then an E873 finding is emitted

  Scenario: E874 — empty alternatives
    Given a TradeStudy with no alternatives
    When the tool validates the model
    Then an E874 finding is emitted

  Scenario: E875 — alternative missing name
    Given an alternatives entry with no name
    When the tool validates the model
    Then an E875 finding is emitted

  Scenario: E876 — score references unknown alternative
    Given a scores entry naming an alternative not in the list
    When the tool validates the model
    Then an E876 finding is emitted

  Scenario: E877 — non-numeric score
    Given a scores entry whose score is not a number
    When the tool validates the model
    Then an E877 finding is emitted

  Scenario: W061 — complete study without decision
    Given a complete TradeStudy with no decision ADR
    When the tool validates the model
    Then a W061 finding is emitted

  Scenario: W063 — incomplete score matrix
    Given a TradeStudy missing an alternative×criterion score
    When the tool validates the model
    Then a W063 finding is emitted

  Scenario: trade-study detail prints a ranked table
    Given the clean model
    When `trade-study TRD-COMM-001` is run
    Then the ranked scoring table is shown

  Scenario: template TradeStudy produces a skeleton
    When the TradeStudy template is printed
    Then it contains `type: TradeStudy`
```

---
id: TC-TRS-CLI-010
type: TestCase
testLevel: L3
status: draft
name: "Verify template works for every element type, uses the current schema, and lists known types from the same source."
verifies:
  - REQ-TRS-CLI-010
---

```gherkin
Feature: element templates (TC-TRS-CLI-010)

  Scenario: every element type has a template
    When the tool runs template for each type in the inventory
    Then each exits 0, except FMEAEntry which points at FMEASheet

  Scenario: the Baseline template carries the baseline create fields
    When the tool runs template Baseline
    Then the skeleton has gitTag, frozenScope and seal

  Scenario: all templates validate together with only placeholder findings
    Given every template rendered into one scratch model
    When the tool validates the model
    Then only unresolved placeholder references, W005/W007 and the allowed coverage warnings remain
    And the StateDef raises no W073/W075 and the TARASheet raises no W030/W032

  Scenario: the unknown-type error lists every known type
    When the tool runs template NoSuchType
    Then it exits non-zero and lists ReviewRecord, TradeStudy, Zone, Conduit and Baseline
```

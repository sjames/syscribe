---
id: TC-TRS-SAFE-006
type: TestCase
testLevel: L3
status: draft
title: Verify W034 freedom-from-interference detection for mixed-criticality shared resources, the FFI-argument exemptions, and the opt-in rule
verifies:
  - REQ-TRS-SAFE-006
---

Verify that the tool flags mixed-criticality elements that share an allocation target
without a freedom-from-interference argument (W034), that an `ffiRationale:` or an
`accepted` `breakdownAdr` clears the finding, that matching integrity levels do not
trigger it, and that `--deny W034` gates.

```gherkin
Feature: Freedom From Interference for mixed-criticality shared resources (W034)

  Scenario: W034 fires on a mixed-criticality sharing without an FFI argument
    Given two software PartDefs of differing ASIL both allocatedTo the same hardware PartDef
    And no element in the pair or the target declares an FFI argument
    When the tool validates the model
    Then at least one W034 finding is emitted naming both sources and their tags
    And the model validates with no errors

  Scenario: an ffiRationale (or accepted breakdownAdr) excuses the sharing
    Given the same shape where the target declares ffiRationale or an accepted breakdownAdr
    When the tool validates the model
    Then no W034 finding is emitted
    And the model validates with no errors

  Scenario: matching integrity levels do not trigger W034
    Given two source elements with the same ASIL allocated to one target
    When the tool validates the model
    Then no W034 finding is emitted

  Scenario: --deny W034 makes validation exit non-zero
    Given the flagged model with an unguarded mixed-criticality sharing
    When the tool validates with --deny W034
    Then the tool exits with a non-zero status
```
</content>

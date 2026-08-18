---
id: TC-TRS-FM-006
type: TestCase
testLevel: L3
status: draft
name: "Verify featureTree: entries auto-derive a FEAT-* id from name when id: is omitted"
verifies:
  - REQ-TRS-FM-006
---

```gherkin
Feature: auto-derived FeatureDef id on a featureTree: entry
  Scenario: a single-segment name derives a simple id
    Given a featureTree: entry { name: Wdt } with no id:
    When the tool runs `validate`
    Then the synthesized FeatureDef Features::Wdt has id: FEAT-WDT
      And no E201 finding is reported

  Scenario: a multi-segment dotted name derives a joined id
    Given a featureTree: entry { name: Platform.CortexM } with no id:
    When the tool runs `validate`
    Then the synthesized FeatureDef Features::Platform::CortexM has id: FEAT-PLATFORM-CORTEXM

  Scenario: an explicit id: always overrides derivation
    Given a featureTree: entry { name: Wdt, id: FEAT-CUSTOM-001 }
    When the tool runs `validate`
    Then the synthesized FeatureDef Features::Wdt has id: FEAT-CUSTOM-001

  Scenario: a grammar-invalid derived id surfaces the existing E006, not a new code
    Given a featureTree: entry { name: X } (a single-character name, no id:)
    When the tool runs `validate`
    Then an E006 finding names the derived id, and the entry is still synthesized

  Scenario: two entries deriving to the same id collide as E101
    Given two featureTree: entries both named "Wdt" in different sheets, neither with id:
    When the tool runs `validate`
    Then an E101 duplicate-id finding is reported

  Scenario: a plain per-file FeatureDef with no id: is unaffected
    Given a per-file FeatureDef with no id: field
    When the tool runs `validate`
    Then E201 is still reported, unchanged
```

---
id: TC-TRS-FM-005
type: TestCase
testLevel: L3
status: draft
name: "Verify single-file, flat/dotted featureTree: sheet — structural tree, crossTreeConstraints:, and parameterConstraints: on FeatureModel"
verifies:
  - REQ-TRS-FM-005
---

```gherkin
Feature: single-file feature model (featureTree:, crossTreeConstraints:, parameterConstraints:)
  Scenario: flat dotted featureTree explodes to the expected qnames and behaves like per-file FeatureDefs
    Given a FeatureModel sheet at Features/_index.md with a mandatory alternative-group entry "Platform"
      And two entries "Platform.CortexM" and "Platform.RiscV"
      And a sibling optional entry "Wdt"
      And a crossTreeConstraints: entry { feature: Wdt, requires: [Platform.CortexM] }
      And a parameterConstraints: entry declared directly on the sheet
      And a Configuration selecting Platform, CortexM, and Wdt
    When the tool runs `validate`
    Then no E-level finding is reported
    When the tool runs `feature-check --deep`
    Then the model is reported sound (no E223 void, no E225 invalid-config)
      And "Features::Platform" appears on the "core features:" line
    When the tool runs `feature-check`
    Then the parameterConstraints: entry on the sheet is evaluated (no E213 unresolved path)

  Scenario: a featureTree entry with no name is dropped and flagged
    Given a FeatureModel sheet with one entry that has no name: field
    When the tool runs `validate`
    Then exactly one E231 finding is reported naming the sheet file

  Scenario: two featureTree entries collide on qname
    Given a FeatureModel sheet whose featureTree has two entries both named "Wdt"
    When the tool runs `validate`
    Then an E232 finding is reported

  Scenario: crossTreeConstraints feature does not resolve within the sheet
    Given a FeatureModel sheet whose crossTreeConstraints: references a feature not defined in that sheet's own featureTree
    When the tool runs `validate`
    Then an E233 finding is reported

  Scenario: featureTree on the wrong element type is inert and flagged
    Given a package _index.md of type: Package that also declares a featureTree: list
    When the tool runs `validate`
    Then a W048 finding is reported
      And no FeatureDef elements are synthesized from that featureTree
```

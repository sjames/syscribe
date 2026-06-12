---
id: TC-TRS-SEC-001
type: TestCase
testLevel: L3
status: draft
name: "Verify safety↔security co-engineering: hazardRef, E844, W030, and the co-analysis view."
verifies:
  - REQ-TRS-SEC-001
---

Verify that `hazardRef` cross-links `DamageScenario`/`ThreatScenario` to a
`HazardousEvent`/`SafetyGoal`, that an unresolved or wrong-type `hazardRef`
produces `E844`, that a safety-tagged `DamageScenario` without `hazardRef`
produces a gateable `W030` while a linked one does not, and that `co-analysis`
(text + `--json`) surfaces the cross-domain chain.

```gherkin
Feature: Safety↔security co-engineering

  Scenario: a linked safety-tagged damage scenario produces no W030 and validates
    Given a model with a SafetyGoal, a safety-tagged DamageScenario linked to it
      via hazardRef, and a ThreatScenario naming that damage scenario
    When the tool validates the model
    Then validation reports zero errors
    And no W030 finding is emitted for the linked damage scenario

  Scenario: a safety-tagged damage scenario with no hazardRef produces W030
    Given the same model with a second safety-tagged DamageScenario that has no hazardRef
    When the tool validates the model
    Then a W030 finding is emitted for the unlinked damage scenario

  Scenario: an unresolved or wrong-type hazardRef produces E844
    Given a DamageScenario whose hazardRef does not resolve
      and a DamageScenario whose hazardRef resolves to a non-safety element
    When the tool validates the model
    Then an E844 finding is emitted

  Scenario: W030 is gateable with --deny
    Given a model with a safety-tagged DamageScenario lacking hazardRef
    When the tool validates with --deny W030
    Then the tool exits non-zero

  Scenario: co-analysis text names the safety goal and its threat
    Given the linked model
    When the tool runs co-analysis
    Then the output names the SafetyGoal and the ThreatScenario that can violate it
    And lists the unlinked safety-tagged damage scenario as a gap

  Scenario: co-analysis --json carries goals and unlinkedSafetyDamage
    Given the linked model
    When the tool runs co-analysis --json
    Then the output is valid JSON with goals and unlinkedSafetyDamage arrays
```

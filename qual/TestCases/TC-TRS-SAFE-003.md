---
id: TC-TRS-SAFE-003
type: TestCase
testLevel: L3
status: draft
title: Verify that DamageScenario and ThreatScenario validation rules E807-E814 and E826 are enforced
verifies:
  - REQ-TRS-SAFE-003
---

Verify that the tool detects all DamageScenario and ThreatScenario field-level and cross-reference validation errors.

```gherkin
Feature: DamageScenario and ThreatScenario validation rules

  Scenario: E807 — missing required field on DamageScenario triggers error
    Given a DamageScenario element missing the id field
    When the tool validates the model
    Then at least one E807 finding is emitted

  Scenario: E808 — DamageScenario id not matching DS-* pattern triggers error
    Given a DamageScenario with id that does not match the DS-* pattern
    When the tool validates the model
    Then at least one E808 finding is emitted

  Scenario: E809 — invalid damageSeverity value triggers error
    Given a valid DamageScenario with damageSeverity set to an invalid value
    When the tool validates the model
    Then at least one E809 finding is emitted

  Scenario: E810 — invalid impactCategories entry triggers error
    Given a valid DamageScenario with an impactCategories entry that is not in the allowed set
    When the tool validates the model
    Then at least one E810 finding is emitted

  Scenario: E811 — missing required field on ThreatScenario triggers error
    Given a ThreatScenario element missing the id field
    When the tool validates the model
    Then at least one E811 finding is emitted

  Scenario: E812 — ThreatScenario id not matching TS-* pattern triggers error
    Given a ThreatScenario with id that does not match the TS-* pattern
    When the tool validates the model
    Then at least one E812 finding is emitted

  Scenario: E813 — invalid attackFeasibility value triggers error
    Given a valid ThreatScenario with attackFeasibility set to an invalid value
    When the tool validates the model
    Then at least one E813 finding is emitted

  Scenario: E814 — invalid attackVector value triggers error
    Given a valid ThreatScenario with attackVector set to an invalid value
    When the tool validates the model
    Then at least one E814 finding is emitted

  Scenario: E826 — unresolvable damageScenarios reference triggers error
    Given a ThreatScenario with a damageScenarios entry that does not resolve to any element
    When the tool validates the model
    Then at least one E826 finding is emitted
```

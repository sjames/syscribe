---
id: TC-TRS-FMA-003
type: TestCase
testLevel: L3
status: draft
title: "Verify anomaly analyses: void, dead, core, false-optional."
verifies:
  - REQ-TRS-FMA-003
---

```gherkin
Feature: Feature model anomaly analyses
  Scenario: dead, core, false-optional detected
    Given a non-void model with a dead feature, a core feature, and a false-optional feature
    When feature-check --deep runs
    Then deadFeatures, coreFeatures, and falseOptionalFeatures list them
    And E224 and W018 are emitted
  Scenario: void dominates
    Given a contradictory model
    Then void is true and no per-feature dead spam is emitted
```

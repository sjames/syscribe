---
id: TC-TRS-FMA-002
type: TestCase
testLevel: L3
status: draft
title: "Verify feature-check --deep command surface, gating, exit codes, and --json."
verifies:
  - REQ-TRS-FMA-002
---

```gherkin
Feature: Deep analysis command surface
  Scenario: --deep is discoverable and gated
    Given the tool
    When --help is printed
    Then feature-check --deep is listed
  Scenario: --deep is opt-in
    Given a void feature model
    When feature-check runs without --deep
    Then no E223 is emitted and it exits 0
  Scenario: exit codes and json keys
    Given feature models
    When feature-check --deep runs
    Then a sound model exits 0, a void model exits 1
    And --json carries void, deadFeatures, coreFeatures, falseOptionalFeatures, invalidConfigurations
  Scenario: dormant with no feature model
    Given a model with no FeatureDef
    Then feature-check --deep prints a notice and exits 0
```

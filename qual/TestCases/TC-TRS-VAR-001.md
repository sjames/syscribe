---
id: TC-TRS-VAR-001
type: TestCase
testLevel: L3
status: draft
title: "Verify that the variability dimension is dormant unless a feature model is linked."
verifies:
  - REQ-TRS-VAR-001
---

Verify the opt-in / dormancy principle: with no feature model, PLE behaviour is absent and variant-only commands fall back gracefully; an unresolved `appliesWhen:` is still an error in all modes.

```gherkin
Feature: Opt-in variability

  Scenario: flat model emits no per-configuration findings
    Given a model with zero FeatureDef elements
    When the tool validates the model
    Then no W015 finding is emitted

  Scenario: matrix on a flat model falls back without error
    Given a model with zero FeatureDef elements
    When the tool runs the matrix command
    Then it prints a "no feature model present" notice
    And it exits with code 0

  Scenario: unresolved appliesWhen is an error even when dormant
    Given a requirement with appliesWhen pointing to a name that does not resolve
    And no FeatureDef elements exist
    When the tool validates the model
    Then an E209 finding is emitted

  Scenario: a feature model with no Configuration does not emit per-config gaps
    Given a FeatureDef and a requirement carrying appliesWhen but no Configuration
    When the tool validates the model
    Then no W015 finding is emitted
```

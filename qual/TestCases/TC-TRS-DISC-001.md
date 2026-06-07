---
id: TC-TRS-DISC-001
type: TestCase
testLevel: L3
status: draft
title: "Verify the `features` command: feature-model overview with groupKind, parameters, and per-feature selection rollup; --json; dormancy."
verifies:
  - REQ-TRS-DISC-001
---

```gherkin
Feature: features command — feature-model overview
  Scenario: features prints the feature model
    Given a product-line model with a feature tree and configurations
    When the tool runs `features`
    Then it exits 0 and prints a "# Feature Model" report
    And every feature qualified name appears
    And the alternative group's groupKind value appears
    And the parameterised feature's parameter name appears
    And each feature shows a "selected in N/M" configuration rollup
  Scenario: machine-readable output
    When the tool runs `features --json`
    Then it exits 0 and emits a JSON document
  Scenario: dormant with no feature model
    Given a model with no feature model
    When the tool runs `features`
    Then it exits 0 and prints a "no feature model" notice
```

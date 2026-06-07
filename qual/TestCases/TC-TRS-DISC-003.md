---
id: TC-TRS-DISC-003
type: TestCase
testLevel: L3
status: draft
title: "Verify `matrix --features`: Feature × Configuration selection grid; default matrix regression."
verifies:
  - REQ-TRS-DISC-003
---

```gherkin
Feature: matrix --features — feature × configuration grid
  Scenario: feature selection grid
    Given a product-line model
    When the tool runs `matrix --features`
    Then it exits 0 and prints a markdown table whose header has "Feature" and the configuration ids
    And a selected feature/config cell shows ✓
  Scenario: default matrix unchanged
    When the tool runs `matrix` with no flag
    Then it still prints the Requirement × Configuration coverage view
```

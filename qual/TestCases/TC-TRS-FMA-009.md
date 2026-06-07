---
id: TC-TRS-FMA-009
type: TestCase
testLevel: L3
status: draft
title: "Verify variant-space count and enumeration."
verifies:
  - REQ-TRS-FMA-009
---

```gherkin
Feature: Variant-space count and enumeration
  Scenario: count of valid configurations
    Given a model with an alternative group of 2 (plus none) and one optional feature
    When the tool runs feature-check --count
    Then variantCount is 6
  Scenario: enumeration lists each valid configuration
    When the tool runs feature-check --enumerate
    Then it lists 6 configurations deterministically
```

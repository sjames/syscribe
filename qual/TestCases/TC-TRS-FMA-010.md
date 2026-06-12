---
id: TC-TRS-FMA-010
type: TestCase
testLevel: L3
status: draft
name: "Verify diagnoses (minimal correction sets) for void models."
verifies:
  - REQ-TRS-FMA-010
---

```gherkin
Feature: Diagnoses for void models
  Scenario: correction sets propose how to fix
    Given a model void due to A requires B and A excludes B
    When feature-check --deep runs
    Then the diagnoses list at least one minimal correction set
    And at least one names the excludes or requires constraint
```

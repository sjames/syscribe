---
id: TC-TRS-PROJ-002
type: TestCase
testLevel: L3
status: draft
name: "Verify full re-validation in the configuration lens."
verifies:
  - REQ-TRS-PROJ-002
---

```gherkin
Feature: Full re-validation in the lens
  Scenario: requirement covered only by an inactive test is uncovered in the variant
    Given REQ-MPS (appliesWhen Mps2) verified by a TestCase appliesWhen Wdt
    When validating with --config the Mps2-without-Wdt configuration
    Then a coverage finding (W002) is reported for REQ-MPS
    And whole-model validate reports no such coverage finding
```

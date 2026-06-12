---
type: TestCase
id: TC-EVCS-004
name: "Verify amortised cost per session is at or below USD 4.00"
status: approved
testLevel: L5
verificationMethod: analysis
verifies:
  - REQ-EVCS-SYS-004
---

## Test Procedure

```gherkin
Feature: Cost per session

  Scenario: Amortised cost at design utilisation
    Given the capex per stall and lifetime session count
    And the per-session energy cost
    When the amortised cost per session is computed
    Then the amortised cost per session is at most USD 4.00
```

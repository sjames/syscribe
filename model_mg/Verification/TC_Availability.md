---
type: TestCase
id: TC-EVCS-003
name: "Verify stall availability meets 99.0% over the analysis window"
status: approved
testLevel: L5
verificationMethod: analysis
verifies:
  - REQ-EVCS-SYS-002
---

## Test Procedure

```gherkin
Feature: Stall availability

  Scenario: Rolling-year availability analysis
    Given the modelled failure and repair rates for a populated cabinet
    When availability is computed over a rolling 12-month window
    Then the predicted stall availability is at least 99.0%
```

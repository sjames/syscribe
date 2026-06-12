---
type: TestCase
id: TC-ENG-SAFE-004
name: HIL — engine stall monitor initiates deceleration on CPS loss
status: active
testLevel: L5
verifies:
  - REQ-ENG-SAFE-003
---

```gherkin
Feature: Engine stall monitor HIL

  Scenario: CPS signal loss at 2000 rpm
    Given the engine is running at 2000 rpm on a running hardware ECU
    When the crankshaft position sensor signal is disconnected
    Then the engine stall monitor detects loss within two engine cycles
    And fuel injection is cut within the next cycle
    And the throttle is commanded to idle position within 50 ms
    And the instrument cluster warning is triggered
    And the engine speed does not drop below 400 rpm before the sequence completes
```

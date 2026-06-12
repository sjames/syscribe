---
type: TestCase
id: TC-ENG-SAFE-002
name: HIL — safety monitor detects all defined fault modes within 100 ms
status: active
testLevel: L5
verifies:
  - REQ-ENG-SAFE-001
---

```gherkin
Feature: Safety monitor fault detection HIL

  Scenario Outline: Single-point fault detection timing
    Given the ECU is running on hardware with fault injection capability
    When <fault_type> is injected via the hardware fault injection harness
    Then the safety monitor sets the fault flag within <max_detection_ms> ms
    And the fail-safe output is asserted within <max_response_ms> ms

    Examples:
      | fault_type                  | max_detection_ms | max_response_ms |
      | TPS track divergence 6 %    | 100              | 200             |
      | throttle stuck at 20 %      | 100              | 200             |
      | pedal-brake conflict        | 100              | 200             |
      | watchdog comm timeout       | 15               | 30              |
```

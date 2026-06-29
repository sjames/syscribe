---
type: TestCase
id: TC-UAV-COMPLIANCE-001
name: "Integrated aircraft conforms to EASA Open Category A3 sub-5 kg limits"
status: active
testLevel: L4
verifies:
  - REQ-UAV-REG-000
tags:
  - regulatory
  - integration
---

System-integration conformance run over the fully integrated, configured aircraft, confirming
the assembled product meets the EASA Open Category A3 airworthiness and operational envelope —
maximum take-off mass and the operating limits required for the sub-5 kg open category.

Run: `cargo xtask hil -- regulatory-conformance --category a3`

```gherkin
Feature: Regulatory conformance integration

  Background:
    Given the UAV is assembled in a flight-ready configuration with maximum payload

  Scenario: The integrated aircraft satisfies the A3 open-category envelope
    When the aircraft take-off mass is measured and the operational limits are checked
    Then the measured take-off mass does not exceed 5 kg
    And the configured operating envelope stays within the A3 open-category limits
    And the compliance record is produced for the airworthiness file
```

---
type: TestCase
id: TC-UAV-COMM-001
name: "Bidirectional data link sustains connectivity at 5 km line of sight"
status: active
testLevel: L5
verifies:
  - REQ-UAV-COMM-001
tags:
  - communication
  - telemetry
  - flight-test
traceBaselines:
  REQ-UAV-COMM-001: "blake3:e5a6b109b67408b0fe44e8a00fcec9cb5df9ab8f3b80ad3ae95b90a2b3c852a2"
---

Field test measuring command uplink and telemetry downlink connectivity at incremental standoff distances up to 5 km. Packet error rate is logged at each distance station.

Run: `cargo xtask hil -- data-link-range-test`

```gherkin
Feature: Data link range verification

  Background:
    Given the UAV is airborne at 50 m AGL
    And the ground station is positioned at the test origin
    And the telemetry logger is recording at 1 Hz

  Scenario: Data link is maintained at 5 km standoff
    When the UAV flies to a waypoint 5 km from the ground station
    Then the bidirectional link shall remain active
    And the packet error rate over 60 seconds shall not exceed 1 %

  Scenario: Command uplink is acknowledged within 500 ms at 5 km
    When a command is issued from the ground station at 5 km standoff
    Then an acknowledgement shall be received within 500 ms
```

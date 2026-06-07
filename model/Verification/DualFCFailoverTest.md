---
type: TestCase
id: TC-UAV-REDUN-001
title: "Backup flight controller assumes control within 100 ms of primary loss"
status: active
testLevel: L4
verifies:
  - REQ-UAV-REDUN-001
appliesWhen: Features::DualFlightController
tags:
  - redundancy
  - fault-injection
---

Fault-injection system test. Disable the primary flight controller mid-flight and
measure failover latency and attitude excursion.

Run: `cargo xtask hil -- failover-test --max-latency-ms 100`

```gherkin
Feature: Dual flight-controller failover

  Scenario: Backup assumes control within budget on primary loss
    Given the UAV is configured with dual flight controllers and is in hover
    When the primary flight controller's heartbeat is lost
    Then the backup flight controller assumes control within 100 ms
    And attitude stabilisation is maintained throughout
```

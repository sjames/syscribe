---
type: TestCase
id: TC-UAV-CARGO-001
title: "Cargo release actuates within 500 ms of validated command"
status: active
testLevel: L4
verifies:
  - REQ-UAV-CARGO-001
appliesWhen: Features::Payload::Delivery
tags:
  - delivery
  - flight-test
---

System test of the cargo delivery payload. Issue a validated release command and
measure actuation latency on the release sensor.

Run: `cargo xtask hil -- cargo-test --max-latency-ms 500`

```gherkin
Feature: Cargo release timing

  Scenario: Release completes within the latency budget
    Given the UAV is configured with the cargo delivery payload
    When a validated cargo-release command is issued
    Then the release mechanism actuates within 500 ms
    And release is confirmed over telemetry
```

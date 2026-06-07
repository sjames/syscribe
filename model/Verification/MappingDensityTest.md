---
type: TestCase
id: TC-UAV-MAP-001
title: "LiDAR mapping payload achieves >= 100 points/m^2 at survey altitude"
status: active
testLevel: L4
verifies:
  - REQ-UAV-MAP-001
appliesWhen: Features::Payload::Mapping
tags:
  - mapping
  - flight-test
---

System flight test of the mapping payload. Fly the nominal survey pattern and
process the captured point cloud; assert ground point density.

Run: `cargo xtask hil -- mapping-test --min-density 100`

```gherkin
Feature: LiDAR mapping point density

  Scenario: Survey pattern yields photogrammetry-grade density
    Given the UAV is configured with the LiDAR mapping payload
    When the UAV flies the nominal survey pattern at survey altitude
    Then the processed point cloud has at least 100 points per square metre
```

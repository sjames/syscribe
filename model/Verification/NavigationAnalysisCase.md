---
type: TestCase
id: TC-UAV-NAV-001
title: "Monte Carlo GNSS simulation demonstrates ≤ 1.5 m CEP"
status: active
testLevel: L2
verifies:
  - REQ-UAV-NAV-001
tags:
  - navigation
  - gps
  - analysis
---

Analytical verification using Monte Carlo simulation (≥ 1000 runs) of GNSS constellation geometry and receiver noise model. Simulation uses STK or equivalent GNSS simulation tool. Atmospheric model: standard tropospheric delay with 50th percentile troposphere.

Run: `cargo test -p nav-analysis -- gps_cep_simulation`

```gherkin
Feature: GPS position accuracy analysis

  Background:
    Given a Monte Carlo simulation with 1000 runs
    And GNSS constellation geometry for the nominal operating area
    And a receiver noise model with standard deviation 0.5 m

  Scenario: CEP is within requirement over 1000 simulation runs
    When the simulation is executed over the full test orbit
    Then the 50th percentile radial position error shall be less than or equal to 1.5 m

  Scenario: At least 6 satellites visible throughout the simulation
    When the simulation is executed
    Then the minimum visible satellite count over all time steps shall be at least 6
```

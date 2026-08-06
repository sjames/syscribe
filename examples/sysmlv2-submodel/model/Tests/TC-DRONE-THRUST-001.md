---
type: TestCase
id: TC-DRONE-THRUST-001
name: "Static thrust stand confirms thrust-to-weight ratio"
status: active
testLevel: L3
verifies:
  - REQ-DRONE-THRUST-001
tags:
  - propulsion
  - thrust
---

Bench test: mount the fully assembled drone on a static thrust stand and confirm measured thrust against maximum takeoff mass meets the required ratio, for each rotor configuration.

```gherkin
Feature: Drone static thrust-to-weight ratio

  Scenario: Thrust-to-weight ratio meets the floor at max takeoff mass
    Given the drone is mounted on the static thrust stand at maximum takeoff mass
    When all rotors are commanded to full throttle
    Then the measured thrust-to-weight ratio shall be at least 2:1
```

---
type: TestCase
id: TC-UAV-MASS-001
title: "UAV total take-off mass does not exceed 5.0 kg on calibrated scale"
status: active
testLevel: L5
verifies:
  - REQ-UAV-MASS-001
tags:
  - mass
  - regulatory
---

Physical weighing of the fully assembled UAV including battery pack and nominal survey payload on a calibrated platform scale (resolution ≤ 0.01 kg, calibration valid within 12 months).

```gherkin
Feature: MTOM compliance

  Background:
    Given a calibrated platform scale with resolution 0.01 kg
    And the UAV is fully assembled with battery and nominal payload installed

  Scenario: Fully assembled mass is within regulatory limit
    When the UAV is placed on the scale
    Then the displayed mass shall be less than or equal to 5.00 kg

  Scenario: Mass reading is repeatable within instrument uncertainty
    When the UAV is weighed three consecutive times
    Then all three readings shall agree within 0.05 kg
```

---
id: TC-HIL-001
type: TestCase
title: "HIL MPU trap test"
status: active
testLevel: L5
verifies: [REQ-HIL-001]
---
```gherkin
Feature: mpu
  Scenario: trap
    Given an out-of-region access
    Then a fault is raised
```

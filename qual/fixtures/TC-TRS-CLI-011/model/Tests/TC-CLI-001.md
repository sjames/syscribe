---
type: TestCase
id: TC-CLI-001
name: "Controller reads the sensor"
status: draft
testLevel: L3
verifies:
  - REQ-CLI-001
---

```gherkin
Feature: controller

  Scenario: read
    Given the sensor
    When the controller polls
    Then it reads a value
```

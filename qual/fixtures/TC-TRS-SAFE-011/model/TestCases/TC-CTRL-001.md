---
id: TC-CTRL-001
type: TestCase
testLevel: L3
status: draft
name: "Verify hard stop limits"
verifies:
  - REQ-CTRL-001
---

```gherkin
Feature: Hard stops

  Scenario: Actuator hard stop
    Given an actuator at limit
    When movement is commanded beyond limit
    Then movement is blocked
```

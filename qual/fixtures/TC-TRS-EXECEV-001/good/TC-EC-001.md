---
type: TestCase
id: TC-EC-001
name: controller
status: active
testLevel: L2
subject: Exec::CtlM
verifies: [REQ-EC-001, REQ-EC-002]
---

```gherkin
Feature: controller

  @verifies REQ-EC-001
  Scenario: go runs
    When Go is injected
    Then CtlM is in state Run

  @verifies REQ-EC-002
  Scenario: stop idles
    When Go is injected
    And Stop is injected
    Then CtlM is in state Idle
```

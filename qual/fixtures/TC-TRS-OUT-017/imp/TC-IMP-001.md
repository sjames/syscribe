---
type: TestCase
id: TC-IMP-001
name: "Verify the 50 ms trip time"
status: active
testLevel: L3
verifies: [REQ-IMP-LEAF-001]
---
```gherkin
Feature: trip time
  Scenario: trips within 50ms
    Given a fault
    Then the detector trips within 50 ms
```

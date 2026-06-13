---
type: TestCase
id: TC-RIF-001
name: "Verify 50ms trip"
status: active
testLevel: L3
verifies:
  - REQ-RIF-LEAF-001
---
```gherkin
Feature: trip
  Scenario: trips
    Then within 50ms
```

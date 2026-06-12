---
id: TC-NRD-CORE-001
type: TestCase
testLevel: L3
status: approved
name: "Test TC-NRD-CORE-001"
verifies:
  - REQ-NRD-CORE-001
---

```gherkin
Feature: TC-NRD-CORE-001
  Scenario: nominal
    Given the system
    Then REQ-NRD-CORE-001 is satisfied
```

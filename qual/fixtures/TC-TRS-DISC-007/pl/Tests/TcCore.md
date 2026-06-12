---
id: TC-D7-CORE-001
type: TestCase
testLevel: L3
status: approved
name: "Test TC-D7-CORE-001"
verifies:
  - REQ-D7-CORE-001
---

```gherkin
Feature: TC-D7-CORE-001
  Scenario: nominal
    Given the system
    Then REQ-D7-CORE-001 is satisfied
```

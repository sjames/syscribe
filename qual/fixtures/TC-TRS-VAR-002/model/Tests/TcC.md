---
id: TC-V2-CORE-001
type: TestCase
testLevel: L3
status: approved
name: "Test TC-V2-CORE-001"
verifies:
  - REQ-V2-CORE-001
---

```gherkin
Feature: TC-V2-CORE-001
  Scenario: nominal
    Given the system
    Then REQ-V2-CORE-001 is satisfied
```

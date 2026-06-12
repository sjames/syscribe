---
id: TC-RDY-CORE-001
type: TestCase
testLevel: L3
status: active
name: "Test TC-RDY-CORE-001"
verifies:
  - REQ-RDY-CORE-001
---

```gherkin
Feature: TC-RDY-CORE-001
  Scenario: nominal
    Given the system
    Then REQ-RDY-CORE-001 is satisfied
```

---
id: TC-P8-CORE-001
type: TestCase
name: "Core supervision test"
status: approved
testLevel: L2
verifies: [REQ-P8-CORE-001]
---
Checks core supervision.

```gherkin
Feature: Supervision

Scenario: core supervision
  Given the system
  Then core supervision holds
```

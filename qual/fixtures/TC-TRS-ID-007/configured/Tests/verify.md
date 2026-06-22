---
id: TC-V-001
type: TestCase
testLevel: L3
status: approved
verifies: [STK-SCHED-001]
name: "verifies a requirement by its configured STK id"
---
```gherkin
Feature: TC-V-001
Scenario: nominal
  Given a configured-prefix requirement
  When the test runs
  Then STK-SCHED-001 shall hold
```

---
id: TC-P8-WDT-001
type: TestCase
name: "Watchdog supervision test"
status: approved
testLevel: L2
verifies: [REQ-P8-CORE-001, REQ-P8-WDT-001]
appliesWhen: Features::Wdt
---
Checks supervision through the watchdog.

```gherkin
Feature: Supervision

Scenario: watchdog supervision
  Given the system
  Then watchdog supervision holds
```

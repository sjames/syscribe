---
id: TC-EX-WDT-001
type: TestCase
testLevel: L3
status: active
name: "Verify watchdog reset within timeoutMs"
appliesWhen: Features::Wdt
verifies:
  - REQ-EX-WDT-001
---

```gherkin
Feature: watchdog timeout
  Scenario: reset fires within timeoutMs
    Given the Wdt feature is selected
    When the watchdog is not serviced for timeoutMs
    Then the system resets
```

---
id: TC-TAG-001
type: TestCase
testLevel: L3
status: approved
name: "Test TC-TAG-001"
verifies:
  - REQ-TAG-001
appliesWhen: Features::Wdt
---

```gherkin
Feature: TC-TAG-001
  Scenario: nominal
    Given the system
    Then REQ-TAG-001 is satisfied
```

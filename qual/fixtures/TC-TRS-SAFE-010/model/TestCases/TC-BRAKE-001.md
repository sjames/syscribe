---
id: TC-BRAKE-001
type: TestCase
testLevel: L3
status: draft
name: "Verify brake requires positive command"
verifies:
  - REQ-BRAKE-001
---

```gherkin
Feature: Brake command

  Scenario: No engagement without positive command
    Given the brake system is armed
    When no positive command is sent
    Then brakes shall not engage
```

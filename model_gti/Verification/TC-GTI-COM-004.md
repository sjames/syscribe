---
type: TestCase
id: TC-GTI-COM-004
name: "Authenticated remote commands"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-053
  - REQ-GTI-STK-005
---

```gherkin
Feature: Command authentication

  Scenario: forged command
    Given a command without valid authentication
    When it is sent
    Then it is rejected and logged

  Scenario: valid command
    Given an authenticated command
    When it is sent
    Then it is accepted
```

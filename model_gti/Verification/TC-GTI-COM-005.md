---
type: TestCase
id: TC-GTI-COM-005
name: "Signed firmware update"
status: active
testLevel: L4
verifies:
  - REQ-GTI-SYS-054
---

```gherkin
Feature: Update

  Scenario: unsigned image
    Given an unsigned image
    When an update is attempted
    Then it is refused

  Scenario: older version
    Given a signed older image
    When an update is attempted
    Then it is refused

  Scenario: valid image
    Given a signed newer image
    When an update is attempted
    Then it installs and boots
```

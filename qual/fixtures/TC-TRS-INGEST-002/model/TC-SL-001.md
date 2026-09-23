---
id: TC-SL-001
type: TestCase
testLevel: L5
status: active
name: "Manually verified via a live session"
verifies: [REQ-SL-001]
---

```gherkin
Feature: manual verification
  Scenario: An empty allowlist denies everything
    Given an empty allowlist
    When a command is issued
    Then it is denied

  Scenario: A configured allowlist permits listed commands
    Given a configured allowlist
    When an allowed command is issued
    Then it succeeds
```

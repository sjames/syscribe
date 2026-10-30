---
id: TC-TRS-BT-030
type: TestCase
testLevel: L3
status: active
name: "Verify params check reports a stale file"
verifies:
  - REQ-TRS-BT-030
---

```gherkin
Feature: Staleness
  Scenario: model changed
    Given a file generated before a parameter's range changed
    When params check is run
    Then a staleness warning is reported and --deny makes it fail
```

---
id: TC-TRS-BT-003
type: TestCase
testLevel: L3
status: active
name: "Verify a malformed [bindingTimes] entry warns and is ignored whole"
verifies:
  - REQ-TRS-BT-003
---

```gherkin
Feature: Malformed binding-time entries
  Scenario: a cyclic order
    Given two stages each declared after the other
    When the project is loaded
    Then one warning is raised and the effective order is the default

  Scenario: two stages in the same position
    Given two stages that would occupy the same gap
    When the project is loaded
    Then both are ignored, each warning naming the other

  Scenario: an unknown key
    Given an entry with an unrecognised key
    When the project is loaded
    Then the entry is warned about and ignored
```

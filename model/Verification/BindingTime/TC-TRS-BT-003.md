---
id: TC-TRS-BT-003
type: TestCase
testLevel: L3
status: draft
name: "Verify a malformed [bindingTimes] entry warns and is ignored whole"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-003.sh
verifies:
  - REQ-TRS-BT-003
---

```gherkin
Feature: Malformed binding-time entries
  Scenario: a cyclic order
    Given two stages each declared after the other
    When the project is loaded
    Then one warning is raised and the effective order is the default

  Scenario: an unknown key
    Given an entry with an unrecognised key
    When the project is loaded
    Then the entry is warned about and ignored
```

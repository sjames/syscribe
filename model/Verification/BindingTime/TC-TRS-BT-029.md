---
id: TC-TRS-BT-029
type: TestCase
testLevel: L3
status: draft
name: "Verify params check validates a values file"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-029.sh
verifies:
  - REQ-TRS-BT-029
---

```gherkin
Feature: params check
  Scenario: out of range
    Given a generated file edited to an out-of-range value
    When params check is run
    Then it fails naming the key and exits non-zero

  Scenario: clean
    Given an unedited generated file
    When params check is run
    Then it passes
```

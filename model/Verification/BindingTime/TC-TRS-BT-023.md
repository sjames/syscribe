---
id: TC-TRS-BT-023
type: TestCase
testLevel: L3
status: active
name: "Verify params export writes the values for a stage keyed by FEAT id"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-023.sh
verifies:
  - REQ-TRS-BT-023
---

```gherkin
Feature: params export
  Scenario: stage selection
    Given a load-stage Configuration
    When params export is run
    Then one entry per load- or runtime-window item and none for compile-only items

  Scenario: keys
    Given the exported file
    When its keys are read
    Then they are FEAT-* ids, not qualified names
```

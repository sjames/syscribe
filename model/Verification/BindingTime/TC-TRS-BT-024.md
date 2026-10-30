---
id: TC-TRS-BT-024
type: TestCase
testLevel: L3
status: draft
name: "Verify the exported file is deterministic and stamped"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-024.sh
verifies:
  - REQ-TRS-BT-024
---

```gherkin
Feature: Export determinism
  Scenario: byte-identical
    Given one model
    When params export is run twice
    Then the files are byte-identical

  Scenario: header
    Given an exported file
    When its first lines are read
    Then they carry the Configuration id, stage and projection hash
```

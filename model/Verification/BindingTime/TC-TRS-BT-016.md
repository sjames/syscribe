---
id: TC-TRS-BT-016
type: TestCase
testLevel: L3
status: draft
name: "Verify residual lists open variability by stage and binder"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-016.sh
verifies:
  - REQ-TRS-BT-016
---

```gherkin
Feature: residual command
  Scenario: compile Configuration
    Given a compile Configuration with load and runtime items open
    When residual --json is run
    Then the load and runtime items are listed and the compile-bound ones are not

  Scenario: determinism
    Given the same model
    When residual --json is run twice
    Then the output is identical and sorted
```

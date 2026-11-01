---
id: TC-TRS-BT-002
type: TestCase
testLevel: L3
status: active
name: "Verify [bindingTimes] declarations are merged into the order"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-002.sh
verifies:
  - REQ-TRS-BT-002
---

```gherkin
Feature: Project-declared stages
  Scenario: a stage placed between two defaults
    Given a deploy stage with after compile and before load
    When binding-times is run
    Then the order is compile, deploy, load, runtime

  Scenario: a stage with no placement
    Given a stage with neither after nor before
    When the project is loaded
    Then the entry is reported unplaceable
```

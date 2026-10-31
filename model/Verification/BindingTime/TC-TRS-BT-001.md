---
id: TC-TRS-BT-001
type: TestCase
testLevel: L3
status: active
name: "Verify the default vocabulary is compile < load < runtime"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-001.sh
verifies:
  - REQ-TRS-BT-001
---

```gherkin
Feature: Default binding-time vocabulary
  Scenario: no project declaration
    Given a project with no bindingTimes table
    When binding-times --json is run
    Then the vocabulary is compile, load, runtime in that order
```

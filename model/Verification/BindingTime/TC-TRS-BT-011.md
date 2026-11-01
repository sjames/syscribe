---
id: TC-TRS-BT-011
type: TestCase
testLevel: L3
status: active
name: "Verify Configuration accepts bindingStage and unstaged is unchanged"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-011.sh
verifies:
  - REQ-TRS-BT-011
---

```gherkin
Feature: bindingStage field
  Scenario: valid and invalid stage
    Given Configurations with bindingStage load and bogus
    When validate is run
    Then load is clean and bogus is an error

  Scenario: unstaged
    Given a Configuration without bindingStage
    When validate is run
    Then the findings are unchanged from before
```

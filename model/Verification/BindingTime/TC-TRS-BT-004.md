---
id: TC-TRS-BT-004
type: TestCase
testLevel: L2
status: draft
name: "Verify no literal binding-time comparison remains outside the BindingTime type"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-004.sh
verifies:
  - REQ-TRS-BT-004
---

```gherkin
Feature: Single BindingTime implementation
  Scenario: structural search of the workspace
    Given the workspace sources
    When ast-grep searches for a literal runtime binding-time comparison and a Some(2) rank test
    Then there are no matches outside the BindingTime type

  Scenario: the existing parameter suite
    Given the BindingTime refactor
    When TC-TRS-PARAM-004 is run
    Then it passes unchanged
```

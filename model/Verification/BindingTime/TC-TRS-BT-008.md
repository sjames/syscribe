---
id: TC-TRS-BT-008
type: TestCase
testLevel: L3
status: draft
name: "Verify a dependent's window may not close before its dependency's opens"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-008.sh
verifies:
  - REQ-TRS-BT-008
---

```gherkin
Feature: Window dependency ordering
  Scenario: parameter derivedFrom a later sibling
    Given a parameter closing at compile derived from a sibling opening at load
    When validate is run
    Then an error is raised; the reverse ordering is clean

  Scenario: feature requires a later feature
    Given a feature requiring one whose window opens after its own closes
    When validate is run
    Then an error is raised

  Scenario: unspecified end
    Given an edge where one end has no window
    When validate is run
    Then the edge is not checked
```

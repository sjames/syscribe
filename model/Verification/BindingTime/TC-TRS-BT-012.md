---
id: TC-TRS-BT-012
type: TestCase
testLevel: L3
status: draft
name: "Verify a staged Configuration binds only within its stage window"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-012.sh
verifies:
  - REQ-TRS-BT-012
---

```gherkin
Feature: Window membership
  Scenario: binding before the window opens
    Given a compile-stage Configuration binding a parameter whose window starts at load
    When validate is run
    Then an error is raised

  Scenario: unstaged runtime binding
    Given an unstaged Configuration binding a runtime parameter
    When validate is run
    Then W027 is still raised
```

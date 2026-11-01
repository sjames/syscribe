---
id: TC-TRS-BT-015
type: TestCase
testLevel: L3
status: active
name: "Verify a closed-window required parameter left unbound is an error"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-015.sh
verifies:
  - REQ-TRS-BT-015
---

```gherkin
Feature: Closed window left unbound
  Scenario: closed window
    Given a compile-window required parameter unbound in a load-stage Configuration
    When validate is run
    Then an error is raised

  Scenario: open window
    Given the same parameter with a load window
    When validate is run
    Then it is residual and silent
```

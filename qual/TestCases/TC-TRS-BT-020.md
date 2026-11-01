---
id: TC-TRS-BT-020
type: TestCase
testLevel: L3
status: active
name: "Verify validate --config treats later-stage items as open"
verifies:
  - REQ-TRS-BT-020
---

```gherkin
Feature: Validation at a stage
  Scenario: later window
    Given a required runtime-window parameter unbound in a compile Configuration
    When validate --config is run
    Then no W017 is raised

  Scenario: same stage
    Given the same parameter with a compile window
    When validate --config is run
    Then W017 is raised
```

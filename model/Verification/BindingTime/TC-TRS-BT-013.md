---
id: TC-TRS-BT-013
type: TestCase
testLevel: L3
status: active
name: "Verify a derived Configuration's stage is not earlier than its base's"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-013.sh
verifies:
  - REQ-TRS-BT-013
---

```gherkin
Feature: Monotonic stage chain
  Scenario: earlier child
    Given a compile Configuration derivedFrom a load Configuration
    When validate is run
    Then an error is raised

  Scenario: later child
    Given a load Configuration derivedFrom a compile Configuration
    When validate is run
    Then no stage error is raised
```

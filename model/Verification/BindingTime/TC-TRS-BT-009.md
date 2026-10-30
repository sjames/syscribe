---
id: TC-TRS-BT-009
type: TestCase
testLevel: L3
status: draft
name: "Verify window ordering is reported by validate and by feature-check"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-009.sh
verifies:
  - REQ-TRS-BT-009
---

```gherkin
Feature: Ordering under validate
  Scenario: both commands
    Given a mis-ordered pair
    When validate and feature-check are each run
    Then each reports it once with no duplicate within one command
```

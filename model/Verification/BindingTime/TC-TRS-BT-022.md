---
id: TC-TRS-BT-022
type: TestCase
testLevel: L3
status: active
name: "Verify provenance appears in show, configure and the JSON outputs"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-022.sh
verifies:
  - REQ-TRS-BT-022
---

```gherkin
Feature: Provenance surfaces
  Scenario: consistent surfaces
    Given a Configuration with inherited and defaulted values
    When show, configure, residual --json and params export --json are run
    Then each lists the same provenance for the same value
```

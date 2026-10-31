---
id: TC-TRS-BT-036
type: TestCase
testLevel: L3
status: draft
name: "Verify the viewer shows windows and stages and filters by stage"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-036.sh
verifies:
  - REQ-TRS-BT-036
---

```gherkin
Feature: Viewer
  Scenario: filter agrees with residual
    Given a staged Configuration in the browser
    When the stage filter is applied
    Then its residual set equals residual --json
```

---
id: TC-TRS-BT-037
type: TestCase
testLevel: L3
status: draft
name: "Verify the editor round-trips the binding-time fields"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-037.sh
verifies:
  - REQ-TRS-BT-037
---

```gherkin
Feature: Editor
  Scenario: round trip
    Given a FeatureDef with unrelated keys
    When a window is set in the editor
    Then the file holds the window and every unrelated key intact
```

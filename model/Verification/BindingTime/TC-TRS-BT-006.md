---
id: TC-TRS-BT-006
type: TestCase
testLevel: L3
status: active
name: "Verify features and parameters accept a binding window and absent means unspecified"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-006.sh
verifies:
  - REQ-TRS-BT-006
---

```gherkin
Feature: Binding windows
  Scenario: a spanning window
    Given a feature with bindingTime compile and latestBindingTime load
    When the model is validated
    Then no finding is raised and the window spans both stages

  Scenario: no window
    Given a feature and a parameter with neither field
    When the model is validated
    Then no binding-time finding of any kind is raised
```

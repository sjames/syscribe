---
id: TC-TRS-BT-031
type: TestCase
testLevel: L3
status: draft
name: "Verify projection retains elements depending on later-bound features"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-031.sh
verifies:
  - REQ-TRS-BT-031
---

```gherkin
Feature: Projection retention
  Scenario: retained at compile
    Given an element applying only when a runtime-window feature is on
    When a compile Configuration is projected
    Then the element is retained and marked variant-retained

  Scenario: removed at runtime
    Given a runtime-stage Configuration that turns the feature off
    When it is projected
    Then the element is removed
```

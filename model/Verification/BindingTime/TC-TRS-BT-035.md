---
id: TC-TRS-BT-035
type: TestCase
testLevel: L3
status: active
name: "Verify matrix --stage gives a per-stage coverage grid"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-035.sh
verifies:
  - REQ-TRS-BT-035
---

```gherkin
Feature: matrix --stage
  Scenario: stage difference
    Given a model with a window spanning load and compile
    When matrix --stage load and matrix --stage compile are run
    Then the grids differ only for items whose window spans both
```

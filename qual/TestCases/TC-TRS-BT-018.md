---
id: TC-TRS-BT-018
type: TestCase
testLevel: L3
status: active
name: "Verify configure --stage reports forced and free choices at a stage"
verifies:
  - REQ-TRS-BT-018
---

```gherkin
Feature: configure --stage
  Scenario: stage filter
    Given a model with items in different windows
    When configure --stage load is run
    Then items whose window excludes load are not listed and the rest agree with residual
```

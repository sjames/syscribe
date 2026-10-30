---
id: TC-TRS-BT-010
type: TestCase
testLevel: L3
status: draft
name: "Verify default-triad single-stage models are unchanged"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-010.sh
verifies:
  - REQ-TRS-BT-010
---

```gherkin
Feature: Backward compatibility of binding time
  Scenario: existing suites
    Given a model using only the default triad and single-stage bindingTime
    When the parameter, placeholder and machine-parameter suites are run
    Then E229, E230, W027, W017, W246 and W248 behave as before
```
